use fbxcel_dom::v7400::object::geometry::TypedGeometryHandle;
use fbxcel_dom::v7400::object::TypedObjectHandle;

/// Returns a useful name for a geometry. Either it's own given name, or the name of the first
/// model that references this geometry.
pub fn geo_name<'a, 'b>(geo: &TypedGeometryHandle<'a>) -> anyhow::Result<&'a str> {
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
