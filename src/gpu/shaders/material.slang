typedef float4 Spectrum;

struct BSDFSample {
	float3 wo;
	float pdf;
};

interface BSDF {
    // Evaluates the bsdf
    Spectrum f(float3 wo,float3 wi);

	Optional<BSDFSample> sample_f(float3 wi);
};

static const float PI = 3.141;

struct DiffuseBSDF : BSDF {
	Spectrum r;	

    Spectrum f(float3 wo,float3 wi) {
		//return r / __BuiltinFloatingPointType.getPi(); 
		return r * 1.0 / PI; 
	}

	Optional<BSDFSample> sample_f(float3 wi) {
		return none;
	}
};
