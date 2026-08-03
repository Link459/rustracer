struct Ray {
	float3 origin;
	float3 direction;
	float t;
}

struct Camera {
	float3 mx;
	float3 my;
	float3 mw;
	float4x4 clip_to_world;

	Ray get_ray(float2 uv) {
		float2 clip_space = uv * 2.0 - 1.0;

		float4 point_ndc = float4(clip_space,0.0f,1.0f);
		//float4 point_world = mul(point_ndc,clip_to_world);
		float4 point_world = mul(clip_to_world,point_ndc);
		float3 orig = point_world.xyz / point_world.w;

		float3 u = fma(-clip_space.x,mw,mx);
		float3 v = fma(-clip_space.y,mw,my);
		float3 d = normalize(cross(u,v));

		return Ray(orig,d,0.0f);
	}
}
