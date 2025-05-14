use std::path::Path;
use std::time::{Duration, Instant};

use image::{AnimationDecoder, DynamicImage, ImageFormat};
use wgpu::{
    AddressMode, Device, Extent3d, FilterMode, Queue, Sampler, SamplerDescriptor, Texture,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

use super::image::ImageTexture;

/// Represents an animated texture with multiple frames
#[derive(Debug)]
pub struct AnimatedTexture {
    /// The individual frames of the animation
    frames: Vec<FrameTexture>,
    /// Current frame index
    current_frame: usize,
    /// Total number of frames
    frame_count: usize,
    /// Whether the animation should loop
    looping: bool,
    /// Last time the frame was updated
    last_update: Instant,
    /// Animation timing accumulator
    time_accumulator: Duration,
    /// The base sampler configuration
    sampler: Sampler,
}

/// Represents a single frame in an animated texture
#[derive(Debug)]
struct FrameTexture {
    /// The texture for this frame
    texture: Texture,
    /// The texture view for rendering
    view: TextureView,
    /// Duration to display this frame
    duration: Duration,
}

impl AnimatedTexture {
    /// Load an animated texture from a path
    pub fn from_path(
        device: &Device,
        queue: &Queue,
        path: impl AsRef<Path>,
        label: &str,
        looping: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        println!("Loading animation from path: {}", path.display());
        let format = ImageFormat::from_path(path)?;
        println!("Detected image format: {format:?}");
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);

        // Create decoder based on format
        let frames = match format {
            ImageFormat::WebP => {
                let decoder = image::codecs::webp::WebPDecoder::new(reader)?;
                let is_animated = decoder.has_animation();
                println!(
                    "WebP file at {} is_animated: {}",
                    path.display(),
                    is_animated
                );

                if !is_animated {
                    // If it's not animated, create a single frame
                    println!("Loading as static image instead of animation");
                    let img =
                        image::load(std::io::BufReader::new(std::fs::File::open(path)?), format)?;
                    return Ok(Self::from_single_image(device, queue, &img, label, looping));
                }

                println!("Attempting to collect animation frames...");
                // Extract frames from animated WebP
                let frames_result = decoder.into_frames().collect::<Result<Vec<_>, _>>();

                match &frames_result {
                    Ok(frames) => {
                        println!("Successfully collected {} animation frames", frames.len())
                    }
                    Err(e) => println!("Error collecting animation frames: {e}"),
                }

                frames_result?
            }
            ImageFormat::Gif => {
                // Process GIF animation
                let decoder = image::codecs::gif::GifDecoder::new(reader)?;
                decoder.into_frames().collect::<Result<Vec<_>, _>>()?
            }
            _ => {
                // For other formats, just load as a single image
                let img = image::load(std::io::BufReader::new(std::fs::File::open(path)?), format)?;
                return Ok(Self::from_single_image(device, queue, &img, label, looping));
            }
        };

        let frame_count = frames.len();
        println!("Loaded {} frames from {}", frame_count, path.display());

        if frame_count == 0 {
            println!("WARNING: No frames loaded from WebP file! Using fallback single image");
            let img = image::load(std::io::BufReader::new(std::fs::File::open(path)?), format)?;
            return Ok(Self::from_single_image(device, queue, &img, label, looping));
        }

        Self::from_frames(device, queue, frames, label, looping)
    }

    /// Create an animated texture from animation frames
    fn from_frames(
        device: &Device,
        queue: &Queue,
        frames: Vec<image::Frame>,
        label: &str,
        looping: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let frame_count = frames.len();

        // Create shared sampler for all frames
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // Process each frame
        let mut frame_textures = Vec::with_capacity(frame_count);
        for (i, frame) in frames.into_iter().enumerate() {
            let frame_label = format!("{label}_{i}");
            let frame_buffer = frame.buffer();
            let (width, height) = frame_buffer.dimensions();

            // Determine frame duration (use a reasonable default if values are extreme)
            let frame_delay = frame.delay().numer_denom_ms();
            println!(
                "Raw frame delay values: {}/{}",
                frame_delay.0, frame_delay.1
            );

            // Check for potential issues with frame delay values
            let duration = if frame_delay.0 == 0 || frame_delay.1 == 0 {
                println!("WARNING: Invalid frame delay! Using default 100ms");
                Duration::from_millis(100)
            } else if (frame_delay.0 as u64 * 1000) / frame_delay.1 as u64 > 10000 {
                // Cap extremely long durations to 500ms
                println!("WARNING: Very long frame duration detected! Capping to 500ms");
                Duration::from_millis(500)
            } else {
                Duration::from_millis((frame_delay.0 as u64 * 1000) / frame_delay.1 as u64)
            };

            // Debug: Print frame duration
            println!(
                "Frame {} duration: {:?} ({}/{}ms)",
                i, duration, frame_delay.0, frame_delay.1
            );

            // Create texture for this frame
            let size = Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&frame_label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            // Write frame data to texture
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                frame_buffer,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                size,
            );

            let view = texture.create_view(&TextureViewDescriptor::default());

            frame_textures.push(FrameTexture {
                texture,
                view,
                duration,
            });
        }

        Ok(Self {
            frames: frame_textures,
            current_frame: 0,
            frame_count,
            looping,
            last_update: Instant::now(),
            time_accumulator: Duration::ZERO,
            sampler,
        })
    }

    /// Create an animated texture from a single static image
    fn from_single_image(
        device: &Device,
        queue: &Queue,
        image: &DynamicImage,
        label: &str,
        looping: bool,
    ) -> Self {
        // Create a regular ImageTexture
        let image_texture = ImageTexture::from_image(device, queue, image, label);

        // Wrap it in an AnimatedTexture with one frame
        let frame = FrameTexture {
            texture: image_texture.texture,
            view: image_texture.view,
            duration: Duration::MAX, // Static image doesn't change
        };

        Self {
            frames: vec![frame],
            current_frame: 0,
            frame_count: 1,
            looping,
            last_update: Instant::now(),
            time_accumulator: Duration::ZERO,
            sampler: image_texture.sampler,
        }
    }

    /// Get the current frame's texture view
    pub fn view(&self) -> &TextureView {
        &self.frames[self.current_frame].view
    }

    /// Get the sampler
    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    /// Update the animation state based on elapsed time
    /// Returns true if the frame changed
    pub fn update(&mut self, dt: Duration) -> bool {
        // Early return if we only have one frame
        if self.frame_count <= 1 {
            println!(
                "No animation: only {} frame, {} total frames in buffer",
                self.frame_count,
                self.frames.len()
            );
            return false;
        }

        self.time_accumulator += dt;
        let old_frame = self.current_frame;

        let frame_duration = self.frames[self.current_frame].duration;
        println!(
            "Animation update: frame {}/{}, time_acc: {:?}, frame_duration: {:?}",
            self.current_frame, self.frame_count, self.time_accumulator, frame_duration
        );

        // DEBUGGING: Force frame advancement every second regardless of frame duration
        let force_advance = self.time_accumulator >= Duration::from_millis(1000);

        if self.time_accumulator >= frame_duration || force_advance {
            // Consume the used time and advance frame
            if force_advance {
                println!("  FORCED FRAME ADVANCEMENT (debug mode)");
                self.time_accumulator = Duration::ZERO;
            } else {
                self.time_accumulator -= frame_duration;
            }

            self.current_frame = (self.current_frame + 1) % self.frame_count;
            println!("  Advancing to frame {}", self.current_frame);

            // If we reached the end and not looping, stay on the last frame
            if !self.looping && self.current_frame == 0 {
                self.current_frame = self.frame_count - 1;
                self.time_accumulator = Duration::ZERO;
                println!(
                    "  Not looping, staying on last frame {}",
                    self.current_frame
                );
            }
        } else {
            println!("  Not enough time accumulated to advance frame");
        }

        // Return true if frame changed
        let changed = old_frame != self.current_frame;
        println!("  Frame changed: {changed}");
        changed
    }

    /// Reset the animation to the first frame
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.time_accumulator = Duration::ZERO;
        self.last_update = Instant::now();
    }

    /// Get the number of frames
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Check if this is an animated texture (has more than one frame)
    pub fn is_animated(&self) -> bool {
        self.frame_count > 1
    }

    /// Check if the animation has finished playing (only relevant when not looping)
    pub fn is_finished(&self) -> bool {
        !self.looping && self.current_frame == self.frame_count - 1
    }

    /// Set whether the animation should loop
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }
}
