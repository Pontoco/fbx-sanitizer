use crate::utils::geo_name;
use fbxcel_dom::v7400::data::mesh::PolygonVertex;
use fbxcel_dom::v7400::object::geometry::TypedGeometryHandle;
use fbxcel_dom::v7400::object::TypedObjectHandle;
use fbxcel_dom::v7400::Document;
use std::collections::HashSet;
use std::iter::FromIterator;

/// Checks for meshes that contain quads or polygons larger than 3 edges. These will be automatically
/// triangulated by Unity on import, but not necessarily the same way your 3D modeling or painting
/// program will do it. This can lead to texture warping when applying the texture in Unity.
///
/// All models should be triangulated before being imported into Substance or Unity.
pub fn verify(doc: &Document) -> anyhow::Result<Vec<String>> {
    let mut errors = vec![];

    for obj in doc.objects() {
        if let TypedObjectHandle::Geometry(geo) = obj.get_typed() {
            if let TypedGeometryHandle::Mesh(m) = geo {
                let polygon_vertices = m.polygon_vertices()?;
                let indices = polygon_vertices.raw_polygon_vertices();

                let mut poly_start_index = 0;
                let indices_len = indices.len();
                let mut poly_sizes = HashSet::<usize>::new();

                while poly_start_index < indices_len {
                    let next_start_index = match indices[poly_start_index..]
                        .iter()
                        .cloned()
                        .map(PolygonVertex::new)
                        .position(PolygonVertex::is_end)
                    {
                        Some(v) => poly_start_index + v + 1,
                        None => anyhow::bail!(
                            "Incomplete polygon found: index_start={:?}, len={}",
                            poly_start_index,
                            indices_len
                        ),
                    };
                    let poly_size = next_start_index - poly_start_index;
                    if poly_size > 3 {
                        poly_sizes.insert(poly_size);
                    }

                    poly_start_index = next_start_index;
                }

                if poly_sizes.len() > 0 {
                    let name = geo_name(&geo).unwrap_or("No Name");
                    let just_quads = [4].iter().cloned().collect();
                    if poly_sizes == just_quads {
                        errors.push(format!(
                            "Mesh [{}] contains quads. \
                            It must be triangulated before importing into Unity.",
                            name
                        ))
                    } else {
                        let sizes = poly_sizes
                            .iter()
                            .map(|p| format!("{}", p))
                            .collect::<Vec<String>>()
                            .join(",");
                        errors.push(format!(
                            "Mesh [{}] is not triangulated. It contains polygons with sizes: {}",
                            name, sizes
                        ))
                    }
                }
            }
        }
    }

    Ok(errors)
}
