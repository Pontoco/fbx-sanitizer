use crate::utils::geo_name;
use fbxcel_dom::v7400::data::mesh::layer::TypedLayerElementHandle;
use fbxcel_dom::v7400::object::geometry::TypedGeometryHandle;
use fbxcel_dom::v7400::object::TypedObjectHandle;
use fbxcel_dom::v7400::Document;

/// Checks to make sure the object has normals. If it does not, Unity will generate its own normals
/// based on the angle of each edge. Usually this is a terrible way to generate normals. Normals
/// should be generated in the modeling program before export.
pub fn verify(doc: &Document) -> anyhow::Result<Vec<String>> {
    let mut errors = vec![];

    for obj in doc.objects() {
        if let TypedObjectHandle::Geometry(geo) = obj.get_typed() {
            if let TypedGeometryHandle::Mesh(m) = geo {
                let mut found_normals = false;

                for layer_elem in m.layers().flat_map(|l| l.layer_element_entries()) {
                    if let TypedLayerElementHandle::Normal(_) = layer_elem.typed_layer_element()? {
                        found_normals = true;
                    }
                }

                if !found_normals {
                    errors.push(format!(
                        "The mesh [{}] does not have vertex normals. Unity will generate bad normals, instead.",
                        geo_name(&geo).unwrap_or("No Name")
                    ));
                }
            }
        }
    }

    Ok(errors)
}
