struct PCData {
	RWTexture2D.Handle final;
};

[shader("compute")]
[numthreads(1,1,1)]
void main(uint3 thread_id : SV_DispatchThreadID, uniform PCData data) {
	float4 color = float4(1.0f,0.0f,1.0f,1.0f);
	data.final.Store(thread_id.xy, color);
}
