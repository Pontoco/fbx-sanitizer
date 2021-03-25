use anyhow::format_err;
use cgmath::{Point3, Vector3};
use fbxcel_dom::v7400::object::geometry::TypedGeometryHandle;
use fbxcel_dom::v7400::object::TypedObjectHandle;
use fbxcel_dom::v7400::Document;
use itertools::Itertools;
use ordered_float::NotNan;
use std::ops::Add;

fn point_to_not_nan(p: Point3<f64>) -> anyhow::Result<Point3<NotNan<f64>>> {
    Ok(Point3::<NotNan<f64>>::new(
        NotNan::<f64>::new(p.x)?,
        NotNan::<f64>::new(p.y)?,
        NotNan::<f64>::new(p.z)?,
    ))
}

fn max(p1: Point3<NotNan<f64>>, p2: Point3<NotNan<f64>>) -> Point3<NotNan<f64>> {
    Point3::new(p1.x.max(p2.x), p1.y.max(p2.y), p1.z.max(p2.z))
}

fn min(p1: Point3<NotNan<f64>>, p2: Point3<NotNan<f64>>) -> Point3<NotNan<f64>> {
    Point3::new(p1.x.min(p2.x), p1.y.min(p2.y), p1.z.min(p2.z))
}

/// Returns a useful name for a geometry. Either it's own given name, or the name of the first
/// model that references this geometry.
fn geo_name<'a, 'b>(geo: &TypedGeometryHandle<'a>) -> anyhow::Result<&'a str> {
    let model_parent = geo
        .destination_objects()
        .filter(|obj| obj.label().is_none())
        .filter_map(|obj| obj.object_handle())
        .filter_map(|obj| match obj.get_typed() {
            TypedObjectHandle::Model(o) => Some(o),
            _ => None,
        })
        .next();

    if let Some(parent) = model_parent {
        Ok(parent.name().unwrap_or("(no name)"))
    } else {
        Ok(geo.name().unwrap_or("(no name)"))
    }
}

fn vec_to_string<T>(v: Vector3<T>) -> String
where
    T: std::fmt::Display,
{
    format!("({}, {}, {})", v.x, v.y, v.z)
}

/// Verifies that a raw mesh geometry is not too small or too large. Small or large models
/// can cause Unity's UV unwrapper to fail.
pub fn verify(doc: &Document) -> anyhow::Result<Option<String>> {
    const MIN_BOUND_SIZE: f64 = 0.0001;
    const MAX_BOUND_SIZE: f64 = 1000.0;
    for obj in doc.objects() {
        if let TypedObjectHandle::Geometry(geo) = obj.get_typed() {
            if let TypedGeometryHandle::Mesh(m) = geo {
                let p = Point3::new(0, 0, 0);
                let max_bound = m
                    .polygon_vertices()?
                    .raw_control_points()?
                    .map(|p| point_to_not_nan(Point3::new(p.x, p.y, p.z)))
                    .fold_ok(
                        point_to_not_nan(Point3::new(f64::MIN, f64::MIN, f64::MIN))?,
                        max,
                    )?;

                let min_bound = m
                    .polygon_vertices()?
                    .raw_control_points()?
                    .map(|p| point_to_not_nan(Point3::new(p.x, p.y, p.z)))
                    .fold_ok(
                        point_to_not_nan(Point3::new(f64::MAX, f64::MAX, f64::MAX))?,
                        min,
                    )?;

                let bounds: Vector3<NotNan<f64>> = max_bound - min_bound;
                if *bounds.x < MIN_BOUND_SIZE
                    || *bounds.y < MIN_BOUND_SIZE
                    || *bounds.z < MIN_BOUND_SIZE
                {
                    return Ok(Some(format!(
                        "The bounds (size) of the mesh [{}] are too small. Meshes must be larger than [{}]. The mesh \
                        bounds are of size {}",
                        geo_name(&geo).unwrap_or("No Name"),
                        MIN_BOUND_SIZE,
                        vec_to_string(bounds)
                    )));
                }
                if *bounds.x > MAX_BOUND_SIZE
                    || *bounds.y > MAX_BOUND_SIZE
                    || *bounds.z > MAX_BOUND_SIZE
                {
                    let mesh_name = m.name().unwrap_or("No Name");
                    return Ok(Some(format!(
                        "The bounds (size) of the mesh [{}] are too big. Meshes must be smaller than [{}]. The mesh \
                        bounds are of size {}",
                        geo_name(&geo).unwrap_or("No Name"),
                        MAX_BOUND_SIZE,
                        vec_to_string(bounds)
                    )));
                }
            }
        }
    }

    return Ok(None);
}
