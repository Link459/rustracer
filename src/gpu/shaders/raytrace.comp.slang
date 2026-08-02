import camera;
import mesh;
import hittable;

struct PCData {
	RWTexture2D.Handle final;
	float2 size;
	Camera camera;
};


[shader("compute")]
[numthreads(1,1,1)]
void main(uint3 thread_id : SV_DispatchThreadID,uniform PCData data) {
	//float2 uv = (float2(thread_id.xy) + 0.5) / data.size;
	float2 uv = (float2(thread_id.xy) + 0.5) / float2(WorkgroupCount().xy);
	//Ray ray = data.camera.get_ray(uv);
	Ray ray = Ray(float3(uv * 2 - 1,0.0),float3(0.0,0.0,1.0),0.0);

	Triangle trongle;
	float z = 1.0f;
	trongle.pos[0] = float3(0.0,-0.5,z);
	trongle.pos[1] = float3(0.5,0.5,z);
	trongle.pos[2] = float3(-0.5,0.5,z);
	Optional<HitPayload> payload = trongle.hit(ray);

	float4 color = float4(uv,0.0f,1.0f);

	if(payload.hasValue) {
		color = float4(0.0,payload.value.t ,0.0f,1.0f);
	} else {
		color = float4(1.0,0.0f,0.0f,1.0f);
	}
	data.final.Store(thread_id.xy, color);
}
