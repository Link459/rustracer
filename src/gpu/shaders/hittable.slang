import material;

struct Ray {
	float3 origin;
	float3 direction;
	float t;
}

struct HitPayload {
	Material material;
	float3 normal;
}

interface Hittable {
    Optional<HitPayload> hit(Ray ray);
}
