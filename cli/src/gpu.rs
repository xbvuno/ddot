use ddot_core::image::Image;
use ddot_core::filter::FilterError;
use std::sync::OnceLock;

struct WgpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

unsafe impl Send for WgpuState {}
unsafe impl Sync for WgpuState {}

static WGPU_STATE: OnceLock<Option<WgpuState>> = OnceLock::new();

struct MapFuture {
    result: std::sync::Arc<std::sync::Mutex<Option<Result<(), wgpu::BufferAsyncError>>>>,
}

impl std::future::Future for MapFuture {
    type Output = Result<(), wgpu::BufferAsyncError>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut guard = self.result.lock().unwrap();
        match guard.take() {
            Some(res) => std::task::Poll::Ready(res),
            None => {
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
        }
    }
}

async fn get_wgpu_state() -> Option<(&'static wgpu::Device, &'static wgpu::Queue)> {
    if WGPU_STATE.get().is_none() {
        let state = init_gpu_state().await;
        let _ = WGPU_STATE.set(state);
    }
    WGPU_STATE.get().and_then(|opt| opt.as_ref().map(|s| (&s.device, &s.queue)))
}

async fn init_gpu_state() -> Option<WgpuState> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("ddot-wgpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        )
        .await
        .ok()?;

    Some(WgpuState { device, queue })
}

#[allow(dead_code)]
pub async fn is_gpu_available() -> bool {
    get_wgpu_state().await.is_some()
}

pub async fn run_gpu(
    shader_source: &str,
    image: &mut Image,
    params_bytes: &[u8],
) -> Result<(), FilterError> {
    let (device, queue) = get_wgpu_state()
        .await
        .ok_or(FilterError::GpuUnavailable)?;

    // 1. Create Shader Module
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ddot-compute-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // 2. Prepare pixel buffers
    let width = image.width;
    let height = image.height;
    let num_pixels = (width * height) as usize;
    let pixels_byte_size = (num_pixels * 4) as wgpu::BufferAddress;

    use wgpu::util::DeviceExt;

    let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ddot-input-buffer"),
        contents: &image.pixels,
        usage: wgpu::BufferUsages::STORAGE,
    });

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ddot-output-buffer"),
        size: pixels_byte_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // 3. Prepare dimensions buffer (Binding 2)
    let dims = [width, height];
    let dims_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ddot-dims-buffer"),
        contents: bytemuck::cast_slice(&dims),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    // 4. Prepare params buffer (Binding 3)
    let has_params = !params_bytes.is_empty();

    let params_buffer = if has_params {
        let mut padded = params_bytes.to_vec();
        while padded.len() % 4 != 0 {
            padded.push(0);
        }
        Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ddot-params-buffer"),
            contents: &padded,
            usage: wgpu::BufferUsages::UNIFORM,
        }))
    } else {
        None
    };

    // 5. Create Bind Group Layout and Bind Group
    let mut entries = vec![
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ];

    if has_params {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    }

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ddot-bind-group-layout"),
        entries: &entries,
    });

    let mut bind_group_entries = vec![
        wgpu::BindGroupEntry {
            binding: 0,
            resource: input_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: output_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 2,
            resource: dims_buffer.as_entire_binding(),
        },
    ];

    if let Some(ref buf) = params_buffer {
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: 3,
            resource: buf.as_entire_binding(),
        });
    }

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ddot-bind-group"),
        layout: &bind_group_layout,
        entries: &bind_group_entries,
    });

    // 6. Create Pipeline Layout and Compute Pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("ddot-pipeline-layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("ddot-compute-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: "main",
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    // 7. Command Encoder & Dispatch
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ddot-encoder"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ddot-compute-pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        let workgroups_x = (width + 15) / 16;
        let workgroups_y = (height + 15) / 16;
        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }

    // 8. Readback staging buffer
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ddot-staging-buffer"),
        size: pixels_byte_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, pixels_byte_size);

    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let result = std::sync::Arc::new(std::sync::Mutex::new(None));
    let result_clone = result.clone();
    buffer_slice.map_async(wgpu::MapMode::Read, move |res| {
        let mut guard = result_clone.lock().unwrap();
        *guard = Some(res);
    });

    device.poll(wgpu::Maintain::Wait);

    MapFuture { result }.await.map_err(|_| FilterError::GpuExecutionFailed)?;

    {
        let data = buffer_slice.get_mapped_range();
        image.pixels.copy_from_slice(&data);
    }

    staging_buffer.unmap();

    Ok(())
}
