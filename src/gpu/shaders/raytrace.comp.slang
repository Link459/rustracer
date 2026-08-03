import camera;
import mesh;
import hittable;

struct PCData {
	RWTexture2D.Handle final;
	float2 size;
	Camera camera;
};

bool hit_sphere(float3 center, float radius, Ray ray) {
	float3 oc = center - ray.origin;
	float a = dot(ray.direction,ray.direction);
	float b = -2.0 * dot(ray.direction,oc);
	float c = dot(oc,oc) - radius * radius;
	float discriminant = b * b - 4 * a * c;
	return discriminant >= 0;
}


[shader("compute")]
[numthreads(16,16,1)]
void main(uint3 thread_id : SV_DispatchThreadID,uniform PCData data) {
	if(thread_id.x >= data.size.x || thread_id.y >= data.size.y) {
		return;
	}

	float2 uv = (float2(thread_id.xy) + 0.5) / data.size;
	Ray ray = data.camera.get_ray(uv);
	float2 viewport_uv = uv * 2.0 - 1.0;
	//Ray ray = Ray(float3(viewport_uv,0.0),float3(0.0,0.0,1.0),0.0);

	Triangle trongle;
	float z = 10.0f;
	trongle.pos[0] = float3(0.0,-0.5,z);
	trongle.pos[1] = float3(-0.5,0.5,z);
	trongle.pos[2] = float3(0.5,0.5,z);
	Optional<HitPayload> payload = trongle.hit(ray);

	float4 color = float4(uv,0.0f,1.0f);

	if(payload.hasValue) {
		color = float4(0.0,payload.value.t ,0.0f,1.0f);
	} else {
		color = float4(1.0,0.0f,0.0f,1.0f);
	}

	data.final.Store(thread_id.xy, color);
}
