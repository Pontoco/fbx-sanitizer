use crate::utils::geo_name;
use fbxcel_dom::v7400::data::mesh::layer::TypedLayerElementHandle;
use fbxcel_dom::v7400::object::geometry::TypedGeometryHandle;
use fbxcel_dom::v7400::object::TypedObjectHandle;
use fbxcel_dom::v7400::Document;

/// Checks to make sure the object has normals. If it does not, Unity will generate its own normals
/// based on the angle of each edge. Usually this is a terrible way to generate normals. Normals
/// should be generated in the modeling program before export.
/// 
/// This check also verifies that the normals found contain the correct number of normals (one for 
/// each vertex). If this is wrong, it will throw errors in Unity on import.
pub fn verify(doc: &Document) -> anyhow::Result<Vec<String>> {
    let mut errors = vec![];

    for obj in doc.objects() {
        if let TypedObjectHandle::Geometry(geo) = obj.get_typed() {
            if let TypedGeometryHandle::Mesh(m) = geo {
                let mesh_name = geo_name(&geo).unwrap_or("No Name");
                let mut found_normals = false;

                // Check for normals and validate their data
                for layer_elem in m.layers().flat_map(|l| l.layer_element_entries()) {
                    if let TypedLayerElementHandle::Normal(_normals) = layer_elem.typed_layer_element()? {
                        found_normals = true;

                        // Validate normal data consistency using low-level node API
                        let geo_node = obj.node();
                        if let Some(normal_node) = geo_node.children_by_name("LayerElementNormal").next() {
                            // Get the normal count from the Normals array
                            if let Some(normals_array_node) = normal_node.children_by_name("Normals").next() {
                                if let Some(normals_attr) = normals_array_node.attributes().get(0) {
                                    if let Some(normal_values) = normals_attr.get_arr_f64() {
                                        let normal_count = normal_values.len() / 3; // 3 components per normal

                                        // Get vertex count from the Vertices array
                                        if let Some(vertices_node) = geo_node.children_by_name("Vertices").next() {
                                            if let Some(vertices_attr) = vertices_node.attributes().get(0) {
                                                if let Some(vertex_values) = vertices_attr.get_arr_f64() {
                                                    let vertex_count = vertex_values.len() / 3; // 3 components per vertex

                                                    // Check mapping mode
                                                    if let Some(mapping_node) = normal_node.children_by_name("MappingInformationType").next() {
                                                        if let Some(mapping_value) = mapping_node.attributes().iter().find_map(|attr| attr.get_string()) {
                                                            // For ByVertice (per-vertex) normals, count should match vertex count
                                                            if mapping_value == "ByVertice" {
                                                                if normal_count != vertex_count {
                                                                    errors.push(format!(
                                                                        "The mesh [{}] has invalid normals: {} normal vectors but {} vertices. For MappingInformationType \"ByVertice\", these counts must match.",
                                                                        mesh_name, normal_count, vertex_count
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !found_normals {
                    errors.push(format!(
                        "The mesh [{}] does not have vertex normals. Unity will generate bad normals, instead.",
                        mesh_name
                    ));
                }
            }
        }
    }

    Ok(errors)
}
