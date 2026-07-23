import model;

struct BVHNode {
}

struct BVH {
	BVHNode* nodes;
	uint32_t* indices;
	Model* models;
}
