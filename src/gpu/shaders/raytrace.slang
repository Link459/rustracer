struct PCData {
	RWTexture2D.Handle final;
};

[shader("compute")]
[numthreads(16,16,1)]
void main(uint3 thread_id : SV_DispatchThreadID, uniform PCData data) {
	float4 color= float4(1.0f,0.0f,1.0f,1.0f);
	data.accum.Store(thread_id.xy, color);
}
