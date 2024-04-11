use rand::Rng;

use crate::{
    hittable::HitPayload,
    ray::Ray,
    texture::{SolidColor, Texture, TextureValue},
    vec3::Vec3,
};

macro_rules! into_mat {
    ($id:ident) => {
        impl Into<Material> for $id {
            fn into(self) -> Material {
                Material::$id(self)
            }
        }
    };
}
#[derive(Clone)]
pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
    DiffuseLight(DiffuseLight),
    Isotropic(Isotropic),
}

impl Material {
    #[inline]
    pub fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        match self {
            Material::Lambertian(ref m) => m.scatter(ray, payload),
            Material::Metal(ref m) => m.scatter(ray, payload),
            Material::Dielectric(ref m) => m.scatter(ray, payload),
            Material::DiffuseLight(ref m) => m.scatter(ray, payload),
            Material::Isotropic(ref m) => m.scatter(ray, payload),
        }
    }

    #[inline]
    pub fn emitted(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        match self {
            Material::DiffuseLight(ref m) => m.emitted(u, v, p),
            _ => Vec3::ZERO,
        }
    }
}

pub trait Scatter: Send + Sync {
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)>;
    fn emitted(&self, _u: f64, _v: f64, _p: &Vec3) -> Vec3 {
        return Vec3::ZERO;
    }
}

#[derive(Clone)]
pub struct Lambertian {
    albedo: Texture,
}

into_mat!(Lambertian);

impl Lambertian {
    pub fn new(albedo: Texture) -> Material {
        return Material::Lambertian(Self { albedo });
    }
}

impl From<Vec3> for Lambertian {
    fn from(value: Vec3) -> Self {
        Self {
            albedo: SolidColor::new(value),
        }
    }
}

impl Scatter for Lambertian {
    #[inline]
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let mut scatter_direction = payload.normal + random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = payload.normal;
        }

        let scattered = Ray::new(payload.p, scatter_direction, ray.time);
        return Some((
            scattered,
            self.albedo.value(payload.u, payload.v, &payload.p),
        ));
    }
}

pub fn random_unit_vector() -> Vec3 {
    return random_unit_sphere().normalize();
}

pub fn random_unit_sphere() -> Vec3 {
    loop {
        let p = Vec3::random(&mut rand::thread_rng(), -1.0..1.0);
        if p.length_squared() >= 1.0 {
            continue;
        }
        return p;
    }
}

#[derive(Clone, Copy)]
pub struct Metal {
    albedo: Vec3,
    fuzz: f64,
}

into_mat!(Metal);

impl Metal {
    pub fn new(albedo: Vec3, fuzz: f64) -> Material {
        return Material::Metal(Self { albedo, fuzz });
    }
}

impl Scatter for Metal {
    #[inline]
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let reflected = ray.dir.normalize().reflect(&payload.normal);
        let scattered = Ray::new(
            payload.p,
            reflected + self.fuzz * random_unit_sphere(),
            ray.time,
        );
        if Vec3::dot(&scattered.dir, &payload.normal) > 0.0 {
            return Some((scattered, self.albedo));
        }

        return None;
    }
}

#[derive(Clone, Copy)]
pub struct Dielectric {
    ir: f64,
}

into_mat!(Dielectric);

impl Dielectric {
    pub fn new(ir: f64) -> Material {
        return Material::Dielectric(Self { ir });
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        // Use Schlick's approximation for reflectance
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Scatter for Dielectric {
    #[inline]
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let refraction_ratio = if payload.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = ray.dir.normalize();

        let cos_theta = ((-1.0) * unit_direction).dot(&payload.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let mut rng = rand::thread_rng();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let will_reflect = rng.gen::<f64>() < Self::reflectance(cos_theta, refraction_ratio);

        let direction = if cannot_refract || will_reflect {
            unit_direction.reflect(&payload.normal)
        } else {
            unit_direction.refract(&payload.normal, refraction_ratio)
        };

        let scattered = Ray::new(payload.p, direction, ray.time);

        return Some((scattered, Vec3::new(1.0, 1.0, 1.0)));
    }
}

#[derive(Clone)]
pub struct DiffuseLight {
    emit: Texture,
}

into_mat!(DiffuseLight);

impl DiffuseLight {
    pub fn new(emit: Texture) -> Material {
        Material::DiffuseLight(Self { emit })
    }
}

impl Scatter for DiffuseLight {
    fn scatter(&self, _ray: &Ray, _payload: &HitPayload) -> Option<(Ray, Vec3)> {
        return None;
    }

    fn emitted(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        return self.emit.value(u, v, &p);
    }
}

#[derive(Clone)]
pub struct Isotropic {
    albedo: Texture,
}

into_mat!(Isotropic);

impl Isotropic {
    pub fn new(texture: Texture) -> Material {
        Material::Isotropic(Self { albedo: texture })
    }
}

impl From<Vec3> for Isotropic {
    fn from(value: Vec3) -> Self {
        Self {
            albedo: SolidColor::new(value),
        }
    }
}

impl Scatter for Isotropic {
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let scattered = Ray::new(payload.p, random_unit_vector(), ray.time);
        let attenuation = self.albedo.value(payload.u, payload.v, &payload.p);
        return Some((scattered, attenuation));
    }
}
