use ash::vk;

struct RVkInstance {
    pub instance: ash::Instance,
}

#[derive(thiserror::Error, Debug)]
pub enum RendererError {
    #[error("Failed to load Video Driver: {0}")]
    FailedToLoadDriver(#[from] ash::LoadingError),
}

pub struct Renderer {
    instance: RVkInstance,
}

impl Renderer {
    pub fn new() -> Result<Renderer, RendererError> {
        let entry = unsafe { ash::Entry::load()? };
        let app_info = vk::ApplicationInfo {
            p_application_name: b"Faisca App\0" as *const u8 as *const i8,
            application_version: vk::make_api_version(0, 1, 0, 0),
            p_engine_name: b"Faisca\0" as *const u8 as *const i8,
            engine_version: vk::make_api_version(0, 1, 0, 0),
            api_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };
        let instance_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };
        todo!()
    }
}