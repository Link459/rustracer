import camera;
import mesh;
import hittable;
import material;

struct PCData {
	RWTexture2D.Handle final;
	float2 size;
	float2 jitter;
	Camera* camera;
	Mesh mesh;
};

bool hit_sphere(float3 center, float radius, Ray ray) {
	float3 oc = center - ray.origin;
	float a = dot(ray.direction,ray.direction);
	float b = -2.0 * dot(ray.direction,oc);
	float c = dot(oc,oc) - radius * radius;
	float discriminant = b * b - 4 * a * c;
	return discriminant >= 0;
}

float4 tonemap(float4 x) {
	float a = 2.51f;
	float b = 0.03f;
	float c = 2.43f;
	float d = 0.59f;
	float e = 0.14f;
	return saturate((x*(a*x+b))/(x*(c*x+d)+e));
}

[shader("compute")]
[numthreads(16,16,1)]
void main(uint3 thread_id : SV_DispatchThreadID,uniform PCData data) {
	if(thread_id.x >= data.size.x || thread_id.y >= data.size.y) {
		return;
	}

	float2 uv = (float2(thread_id.xy) + 0.5) / data.size + data.jitter;
	//Ray ray = data.camera.get_ray(uv);
	Ray ray = data.camera.get_ray_o(uv);
	//Ray ray = Ray(float3(uv * 2 - 1,0.0),float3(0.0,0.0,1.0),0.0);

	Triangle trongle;
	float z = 10.0f;
	trongle.pos[0] = float3(0.0,-0.5,z);
	trongle.pos[1] = float3(-0.5,0.5,z);
	trongle.pos[2] = float3(0.5,0.5,z);
	//Optional<HitPayload> payload = trongle.hit(ray);
	Optional<HitPayload> payload = data.mesh.hit(ray);
	BSDF bsdf = DiffuseBSDF({0.0f,1.0f,0.0f,1.0f});

	float4 color = float4(uv,0.0f,1.0f);

	if(payload.hasValue) {
		color = bsdf.f(ray.direction,ray.direction);
	} else {
		color = float4(1.0,0.0f,0.0f,1.0f);
	}
	color = tonemap(color);

	data.final.Store(thread_id.xy, color);
}
