struct Rng {
	uint state;

	[mutating]
	uint next_random() {
		state = state * 747796405 + 2891336453;
    	uint result = ((state >> ((state >> 28) + 4)) ^ state) * 277803737;
    	result = (result >> 22) ^ result;
   		return result;
	}

	[mutating]
	float random() {
		return next_random() / (2 ^ 32 - 1);
	}
}
