use bevy_ecs::prelude::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsaaSampleCount {
    X1 = 1,
    X2 = 2,
    X4 = 4,
    X8 = 8,
}

impl MsaaSampleCount {
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::X1),
            2 => Some(Self::X2),
            4 => Some(Self::X4),
            8 => Some(Self::X8),
            _ => None,
        }
    }
}

impl Default for MsaaSampleCount {
    fn default() -> Self {
        Self::X1
    }
}

#[derive(Debug, Clone, Resource)]
pub struct GraphicsSettings {
    msaa_sample_count: MsaaSampleCount,
    changed: bool,
}

impl GraphicsSettings {
    pub fn new(msaa_sample_count: MsaaSampleCount) -> Self {
        Self {
            msaa_sample_count,
            changed: true,
        }
    }

    pub fn msaa_sample_count(&self) -> MsaaSampleCount {
        self.msaa_sample_count
    }

    pub fn set_msaa_sample_count(&mut self, count: MsaaSampleCount) {
        if self.msaa_sample_count != count {
            self.msaa_sample_count = count;
            self.changed = true;
        }
    }

    pub fn take_changed(&mut self) -> bool {
        let changed = self.changed;
        self.changed = false;
        changed
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self::new(MsaaSampleCount::default())
    }
}
