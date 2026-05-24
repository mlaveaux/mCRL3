use merc_utilities::MercError;
use wgpu::Instance;

/// Initializes wgpu and returns the device and queue.
pub async fn init_wgpu() -> Result<(wgpu::Device, wgpu::Queue), MercError> {
    let instance = Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .map_err(|e| MercError::from(format!("Cannot find a suitable adapter: {e}")))?;

    adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .map_err(|e| MercError::from(format!("Failed to create device: {e}")))
}
