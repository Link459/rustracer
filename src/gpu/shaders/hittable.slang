import material;
import camera;

struct HitPayload {
	//BSDF material;
	float3 normal;
	float3 pos;
	float t;
}

interface Hittable {
    Optional<HitPayload> hit(Ray ray);
}
