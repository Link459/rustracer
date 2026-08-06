import hittable;
import camera;

static const uint CW_FIRST_TRIANGLE_INDEX = 0;
static const uint CW_LAST_TRIANGLE_INDEX = 2;

static const uint CCW_FIRST_TRIANGLE_INDEX = 2;
static const uint CCW_LAST_TRIANGLE_INDEX = 0;


static const uint FIRST_TRIANGLE_INDEX = CW_FIRST_TRIANGLE_INDEX;
static const uint LAST_TRIANGLE_INDEX = CW_LAST_TRIANGLE_INDEX;

//static const uint FIRST_TRIANGLE_INDEX = CCW_FIRST_TRIANGLE_INDEX;
//static const uint LAST_TRIANGLE_INDEX = CCW_LAST_TRIANGLE_INDEX;

struct Triangle : Hittable {
	float3 pos[3];
	//float3 normal[3];


    Optional<HitPayload> hit(Ray ray) {
		float3 edge1 = pos[1] - pos[FIRST_TRIANGLE_INDEX];
		float3 edge2 = pos[LAST_TRIANGLE_INDEX] - pos[FIRST_TRIANGLE_INDEX];

		float3 normal = cross(edge1,edge2);
		if(dot(normal,ray.direction) > 0) {
			return none;
		}

		float det = -dot(ray.direction, normal);
		if (det <= 1e-8) {
			return none;
		}

		float inv_det = 1.0f / det;

		float3 vert_ray_offset = ray.origin - pos[FIRST_TRIANGLE_INDEX];
		float3 ray_offset_perp = cross(ray.direction,vert_ray_offset);
		float dst = dot(vert_ray_offset,normal); 


		float u = dot(ray_offset_perp, edge2) * inv_det;
		float v = -dot(ray_offset_perp, edge1) * inv_det;
		float w = 1 - u - v;

		if(!(dst > 0 && u >= 0 && v >= 0 && w >= 0)) {
			return none;
		}
		
		//float3 smooth_normal = normalize(normal[0] * w + normal[1] * u + normal[2] * v);
		float3 hit_point = ray.origin + ray.direction * dst;
		return HitPayload(normal,hit_point,dst);
	}
};


struct Mesh : Hittable {
	Triangle* triangles;
	uint* indices;
	uint index_count;

    Optional<HitPayload> hit(Ray ray) {
		for(uint i = 0; i < index_count; i++) {
			let rtriangle = triangles[i];

			//let hit = triangles[indices[i]].hit(ray);
			let hit = rtriangle.hit(ray);
			if(hit.hasValue) {
				return hit;
			}
		}
		return none;
	}
};
