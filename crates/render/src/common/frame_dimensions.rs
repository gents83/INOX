//! Authoritative dimensions for one rendered frame.

const WORKGROUP_SIZE: u32 = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Extent2d {
    pub width: u32,
    pub height: u32,
}

impl Extent2d {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn pixel_count(self) -> Result<u32, FrameDimensionsError> {
        self.width
            .checked_mul(self.height)
            .ok_or(FrameDimensionsError::PixelCountOverflow {
                width: self.width,
                height: self.height,
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameDimensionsError {
    PixelCountOverflow { width: u32, height: u32 },
    DispatchOverflow { width: u32, height: u32 },
}

/// Whether render-scale changes storage/dispatch dimensions or only reconstruction quality.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderScalePolicy {
    ReconstructionOnly,
    ScaleRenderExtent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameDimensions {
    pub surface: Extent2d,
    pub render: Extent2d,
    /// Number of pixels between rows in full-resolution storage buffers.
    pub storage_stride: u32,
    /// Workgroup counts for the render extent, not pixel counts.
    pub dispatch: [u32; 2],
}

impl FrameDimensions {
    pub fn new(width: u32, height: u32, render_scale: f32) -> Result<Self, FrameDimensionsError> {
        Self::with_policy(
            width,
            height,
            render_scale,
            RenderScalePolicy::ReconstructionOnly,
        )
    }

    pub fn with_policy(
        width: u32,
        height: u32,
        render_scale: f32,
        policy: RenderScalePolicy,
    ) -> Result<Self, FrameDimensionsError> {
        let surface = Extent2d::new(width.max(1), height.max(1));
        // NaN must not reach float-to-integer conversion. Reconstruction-only is
        // the production policy, so invalid scale values simply mean native size.
        let scale = if render_scale.is_finite() {
            render_scale.clamp(0.25, 1.0)
        } else {
            1.0
        };
        let render = match policy {
            RenderScalePolicy::ReconstructionOnly => surface,
            RenderScalePolicy::ScaleRenderExtent => Extent2d::new(
                scaled_extent(surface.width, scale),
                scaled_extent(surface.height, scale),
            ),
        };
        surface.pixel_count()?;
        render.pixel_count()?;
        let dispatch = [
            dispatch_for(render.width, render.height)?,
            dispatch_for(render.height, render.width)?,
        ];
        Ok(Self {
            surface,
            render,
            storage_stride: surface.width,
            dispatch,
        })
    }

    pub const fn render_dimensions(self) -> (u32, u32) {
        (self.render.width, self.render.height)
    }

    /// Number of mip levels required to cover the full-resolution extent.
    pub fn hiz_mip_count(self) -> u32 {
        let mut largest = self.surface.width.max(self.surface.height);
        let mut count = 1;
        while largest > 1 {
            largest /= 2;
            count += 1;
        }
        count
    }

    pub fn storage_pixel_count(self) -> Result<u32, FrameDimensionsError> {
        self.storage_stride.checked_mul(self.surface.height).ok_or(
            FrameDimensionsError::PixelCountOverflow {
                width: self.storage_stride,
                height: self.surface.height,
            },
        )
    }
}

fn scaled_extent(value: u32, scale: f32) -> u32 {
    ((value as f64 * f64::from(scale)).ceil() as u32).max(1)
}

fn dispatch_for(width: u32, height: u32) -> Result<u32, FrameDimensionsError> {
    width
        .checked_add(WORKGROUP_SIZE - 1)
        .map(|value| value / WORKGROUP_SIZE)
        .ok_or(FrameDimensionsError::DispatchOverflow { width, height })
}
