import material;

struct Ray {
	float3 origin;
	float3 direction;
	float t;
}

struct HitPayload {
	BSDF material;
	float3 normal;
}

interface Hittable {
    Optional<HitPayload> hit(Ray ray);
}
