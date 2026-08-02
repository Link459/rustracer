import mesh;
import camera;
import hittable;

struct BVHNode {
	float3 min_bounds;
	float3 max_bounds;
	uint primitive_count;
	uint first_idx;

	bool is_leaf() {
		return primitive_count != 0;
	}

	bool hit(Ray ray) {
		return true;
	}
};

struct BVH : Hittable {
	BVHNode* nodes;
	uint32_t* indices;
	Mesh* models;

    Optional<HitPayload> hit(Ray ray) {
		uint stack[32];
		stack[0] = 0;
		uint stack_count = 1;

        while (stack_count > 0){
            let node = &nodes[stack[stack_count--]];

            if (!node.hit(ray)) {
                continue;
            }

            if (node.is_leaf()) {
                // we keep track of the most current hit and check the distance to ensure that only
                // the primitive nearest to the camera is returned

				Optional<HitPayload> current = none;
				for (uint i = 0; i < node.primitive_count; i++) {
					let prim_index = indices[node.first_idx + i];
					let x = models[prim_index].hit(ray);
					if (x.hasValue) {
						if(current.hasValue) {
							if(x.value.t < current.value.t) {
								current = x;
							}
						}
						else {
							current = x;
                        }
					}
				}
                /*let mut current: Option<(HitPayload, MaterialId)> = None;

                for i in 0..node.primitive_count {
                    let prim_index = self.prim_indices[(node.first_idx + i) as usize];

                    let model = &self.models[prim_index];

                    if let Some(x) = model.hit(ray, ray_t) {
                        if let Some(ref y) = current {
                            if x.0.t < y.0.t {
                                current = Some(x);
                            }
                        } else {
                            current = Some(x);
                        }
                    }
                }

                return current;*/
				return none;
            } else {
				stack[stack_count++] = node.first_idx;
				stack[stack_count++] = node.first_idx + 1;
                //stack.push_back(node.first_idx as usize);
                //stack.push_back(node.first_idx as usize + 1);
            }
        }
		return none;
    }

}
