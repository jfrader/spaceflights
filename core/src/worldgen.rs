use crate::Seed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldProfilePreset {
    Main,
    DevDebug,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StarLayerProfile {
    pub count: u32,
    pub depth_min_km: f64,
    pub depth_max_km: f64,
    pub parallax: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldProfile {
    pub label: &'static str,
    pub world_extent_km: f64,
    pub star_layers: [StarLayerProfile; 3],
    pub horizon_debris_count: u32,
    pub horizon_band_km: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StarPoint {
    pub position: Vec3d,
    pub brightness: f32,
    pub parallax: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizonDebris {
    pub position: Vec3d,
    pub drift: Vec3d,
    pub parallax: f64,
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldSnapshot {
    pub seed: Seed,
    pub profile: WorldProfile,
    pub stars: Vec<StarPoint>,
    pub horizon_debris: Vec<HorizonDebris>,
}

#[must_use]
pub fn profile_from_preset(
    preset: WorldProfilePreset,
    star_density_percent: u16,
    debris_density_percent: u16,
) -> WorldProfile {
    let (label, extent, base_layers, base_debris, band) = match preset {
        WorldProfilePreset::Main => (
            "main",
            12_000.0,
            [
                StarLayerProfile {
                    count: 320,
                    depth_min_km: -2_500.0,
                    depth_max_km: 2_500.0,
                    parallax: 0.15,
                },
                StarLayerProfile {
                    count: 220,
                    depth_min_km: -6_000.0,
                    depth_max_km: 6_000.0,
                    parallax: 0.07,
                },
                StarLayerProfile {
                    count: 140,
                    depth_min_km: -10_000.0,
                    depth_max_km: 10_000.0,
                    parallax: 0.03,
                },
            ],
            28,
            450.0,
        ),
        WorldProfilePreset::DevDebug => (
            "dev_debug",
            8_500.0,
            [
                StarLayerProfile {
                    count: 240,
                    depth_min_km: -2_000.0,
                    depth_max_km: 2_000.0,
                    parallax: 0.17,
                },
                StarLayerProfile {
                    count: 160,
                    depth_min_km: -4_500.0,
                    depth_max_km: 4_500.0,
                    parallax: 0.08,
                },
                StarLayerProfile {
                    count: 110,
                    depth_min_km: -8_000.0,
                    depth_max_km: 8_000.0,
                    parallax: 0.035,
                },
            ],
            20,
            350.0,
        ),
    };

    let star_scale = f64::from(star_density_percent.max(10)) / 100.0;
    let debris_scale = f64::from(debris_density_percent.max(10)) / 100.0;

    let mut layers = base_layers;
    for layer in &mut layers {
        layer.count = scaled_count(layer.count, star_scale);
    }

    WorldProfile {
        label,
        world_extent_km: extent,
        star_layers: layers,
        horizon_debris_count: scaled_count(base_debris, debris_scale),
        horizon_band_km: band,
    }
}

#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    reason = "World snapshot stores brightness/scale as f32 for rendering payload."
)]
pub fn generate_world(seed: Seed, profile: WorldProfile) -> WorldSnapshot {
    let mut stars: Vec<StarPoint> = Vec::new();
    let mut horizon_debris: Vec<HorizonDebris> = Vec::new();

    let mut index = 0_u64;
    for layer in &profile.star_layers {
        for _ in 0..layer.count {
            let x = range(
                seed,
                index,
                -profile.world_extent_km,
                profile.world_extent_km,
            );
            let y = range(
                seed,
                index + 1,
                -profile.world_extent_km,
                profile.world_extent_km,
            );
            let z = range(seed, index + 2, layer.depth_min_km, layer.depth_max_km);
            let brightness = range(seed, index + 3, 0.45, 1.0) as f32;

            stars.push(StarPoint {
                position: Vec3d { x, y, z },
                brightness,
                parallax: layer.parallax,
            });

            index += 11;
        }
    }

    for i in 0..profile.horizon_debris_count {
        let base = 10_000_u64 + u64::from(i) * 13;
        let x = range(
            seed,
            base,
            -profile.world_extent_km,
            profile.world_extent_km,
        );
        let y = range(
            seed,
            base + 1,
            -profile.horizon_band_km,
            profile.horizon_band_km,
        );
        let z = range(
            seed,
            base + 2,
            -profile.world_extent_km,
            profile.world_extent_km,
        );
        let drift_x = range(seed, base + 3, -2.0, 2.0);
        let drift_y = range(seed, base + 4, -0.8, 0.8);
        let drift_z = range(seed, base + 5, -12.0, -3.0);
        let parallax = range(seed, base + 6, 0.2, 0.45);
        let scale = range(seed, base + 7, 0.5, 2.5) as f32;

        horizon_debris.push(HorizonDebris {
            position: Vec3d { x, y, z },
            drift: Vec3d {
                x: drift_x,
                y: drift_y,
                z: drift_z,
            },
            parallax,
            scale,
        });
    }

    WorldSnapshot {
        seed,
        profile,
        stars,
        horizon_debris,
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Scaled counts are bounded and clamped before conversion."
)]
fn scaled_count(base: u32, scale: f64) -> u32 {
    (f64::from(base) * scale).round().max(1.0) as u32
}

fn range(seed: Seed, index: u64, min: f64, max: f64) -> f64 {
    let t = unit(seed, index);
    min + (max - min) * t
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Deterministic normalization from u64 space to unit interval."
)]
fn unit(seed: Seed, index: u64) -> f64 {
    let value = seed.nth_value(index);
    (value as f64) / (u64::MAX as f64)
}

#[cfg(test)]
mod tests {
    use super::{generate_world, profile_from_preset, WorldProfilePreset};
    use crate::Seed;

    #[test]
    fn generation_is_stable_for_same_seed_and_profile() {
        let seed = Seed::new(42);
        let profile = profile_from_preset(WorldProfilePreset::Main, 100, 100);

        let a = generate_world(seed, profile);
        let b = generate_world(seed, profile);

        assert_eq!(a, b);
    }

    #[test]
    fn generation_differs_across_seeds() {
        let profile = profile_from_preset(WorldProfilePreset::Main, 100, 100);
        let a = generate_world(Seed::new(1), profile);
        let b = generate_world(Seed::new(2), profile);

        assert_ne!(a.stars[0], b.stars[0]);
    }

    #[test]
    fn density_controls_snapshot_size() {
        let seed = Seed::new(99);

        let sparse = profile_from_preset(WorldProfilePreset::Main, 30, 30);
        let dense = profile_from_preset(WorldProfilePreset::Main, 200, 200);

        let sparse_world = generate_world(seed, sparse);
        let dense_world = generate_world(seed, dense);

        assert!(dense_world.stars.len() > sparse_world.stars.len());
        assert!(dense_world.horizon_debris.len() > sparse_world.horizon_debris.len());
    }
}
