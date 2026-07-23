struct PCData {
	uint frame;
	Texture2D.Handle src;
	RWTexture2D.Handle accum;
};

[shader("compute")]
[numthreads(16,16,1)]
void main(uint3 thread_id : SV_DispatchThreadID, uniform PCData data) {
	float4 current_color = data.src.Load(uint3(thread_id.xy,0));
	float4 prev_color = data.accum.Load(thread_id.xy);

    float weight = 1.0 / (data.frame + 1);
    float4 accumulated = prev_color * (1.0 - weight) + current_color * weight;
	data.accum.Store(thread_id.xy,accumulated);
}
