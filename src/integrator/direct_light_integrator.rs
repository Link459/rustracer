use crate::{
    camera::Camera,
    hittable::{Hittable, HittableExt},
    integrator::Integrator,
    interval::Interval,
    light::{LightSampleContext, LightStore, UniformLightSampler},
    material::MaterialStore,
    ray::Ray,
    render::RenderSettings,
    vec3::Vec3,
    Float,
};

pub struct DirectLightingIntegrator<'world, W> {
    camera: Camera,
    world: &'world W,
    lights: UniformLightSampler,
    materials: MaterialStore,
    config: RenderSettings,
}

impl<'world, W> DirectLightingIntegrator<'world, W>
where
    W: Hittable,
{
    pub fn new(
        camera: Camera,
        world: &'world W,
        lights: LightStore,
        materials: MaterialStore,
        config: RenderSettings,
    ) -> Self {
        Self {
            camera,
            world,
            lights: UniformLightSampler::new(lights),
            materials,
            config,
        }
    }

    fn li(&self, mut _ray: Ray, mut _depth: u32) -> Vec3 {
        let incomingLight = Vec3::ZERO;
        let rayColour = Vec3::ONE;

        let max_bounce_count = 50;
        let mut orig = _ray.orig;
        let mut dir = _ray.dir;
        for i in 0..max_bounce_count - 1 {
            let ray = Ray::new(_ray.orig, _ray.dir, _ray.time);
            let ray_t = Interval::new(0.001, Float::INFINITY);
            let Some((payload, material)) = self.world.hit(&ray, ray_t) else {
                //incomingLight += GetEnvironmentLight(rayDir) * rayColour;
                break;
            };

            let material = self.materials.get(material);

            orig = payload.p;
            let Some(scatter) = material.scatter(&dir, &payload) else {
                break;
            };
            dir = scatter.wo;

            let direct_light = Vec3::ZERO;

            if let Some(sampled_light) = self.lights.sample() {
                let ctx = LightSampleContext {
                    p: payload.p,
                    n: payload.normal,
                };

                if let Some(sample) = sampled_light.light.sample_li(&ctx) {
                    let wo = sample.wo;
                    //let f = material.f(wi, wo) * wo.dot(&ctx.n).abs();

                    if self.world.unoccluded(payload.p, sample.p) {
                        //l += (beta * f * sample.l) / (sampled_light.p * sample.pdf);
                    }
                }
            }
            /*// Direct Light Sampling:
            float3 directLight = float3(0.0f, 0.0f, 0.0f);
            float3 w_i_dl;
            float pdf_dl;
            {
                float3 lightPos, normal;
                float V = 1.0f;
                SampleDirectLight(lightPos, normal, pdf_dl, rngState);

                Ray dL_ray;
                dL_ray.origin = rayOrigin;
                w_i_dl = normalize(lightPos - rayOrigin);
                dL_ray.dir = w_i_dl;

                // Todo IMPORTANT EXPLAIN THESE EFFECTS
                if(dot(hitInfo.normal, dL_ray.dir) <= 0.001f){
                    V = 0.0f;
                    //pdf = 1.0f;
                }

                ModelHitInfo dL_hitInfo = CalculateRayCollision(dL_ray, stats);

                // sampled point is not visible
                if(length(dL_hitInfo.hitPoint-lightPos) > 0.001f){
                    V = 0.0f;
                    //pdf = 1.0f;
                }

                float dotA = max(0.001f, abs(dot(normal, normalize(lightPos - rayOrigin))));
                float dotB = max(0.001f, abs(dot(hitInfo.normal, normalize(lightPos - rayOrigin))));
                float G = 1.0f/ (length(rayOrigin-lightPos)*length(rayOrigin-lightPos)) * dotA * dotB;
                directLight = G * V * dL_hitInfo.material.emissionColour * dL_hitInfo.material.emissionStrength /pdf_dl;
            }

            float cosTheta = max(0.001f, dot(hitInfo.normal, rayDir));

            // Update light calculations
            if(material.emissionStrength > 0.0f){
                float3 emittedLight = material.emissionColour * material.emissionStrength;
                if(bounceIndex == 0){
                    incomingLight += emittedLight * rayColour;
                }

                // terminate
                break;
            }
            else {
                float3 brdf =  EvaluateMaterialPhong(rayDir, hitInfo.normal,  ray.dir, material, rngState);

                incomingLight += directLight * rayColour *  EvaluateMaterialPhong(w_i_dl, hitInfo.normal, ray.dir, material, rngState);

                rayColour *= brdf * cosTheta/ pdf_brdf;
            }*/
        }

        return incomingLight;
    }
}

impl<'world, W> Integrator for DirectLightingIntegrator<'world, W>
where
    W: Hittable,
{
    //fn pixel(&self, ray: &Ray, sampler: &dyn Sampler) -> Vec3 {
    fn pixel(&self, ray: &Ray) -> Vec3 {
        return self.li(*ray, 0);
    }

    fn name() -> &'static str {
        return "DirectLightingIntegrator";
    }
}
