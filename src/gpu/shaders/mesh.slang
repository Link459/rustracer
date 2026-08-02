import hittable;
import camera;

struct Triangle : Hittable {
	float3 pos[3];
	//float3 normal[3];


    Optional<HitPayload> hit(Ray ray) {
		float3 edge1 = pos[1] - pos[0];
		float3 edge2 = pos[2] - pos[0];

		float3 normal = cross(edge1,edge2);
		if(dot(normal,ray.direction) > 0) {
			return none;
		}

		float det = -dot(ray.direction, normal);
		if (abs(det) < 0.001) {
			return none;
		}

		float inv_det = 1.0f / det;

		float3 vert_ray_offset = ray.origin - pos[0];
		float3 ray_offset_perp = cross(ray.direction,vert_ray_offset);
		float dst = dot(vert_ray_offset,normal); 
		float u = dot(vert_ray_offset, edge2) * inv_det;
		float v = -dot(vert_ray_offset, edge1) * inv_det;
		float w = 1 - u - v;
		
		//float3 smooth_normal = normalize(normal[0] * w + normal[1] * u + normal[2] * v);
		float3 hit_point = ray.origin + ray.direction * dst;
		return HitPayload(normal,hit_point,dst);
	}
};

struct Model {
};
