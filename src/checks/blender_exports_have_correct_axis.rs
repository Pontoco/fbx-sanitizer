use anyhow::format_err;
use cgmath::Vector3;
use fbxcel_dom::fbxcel::low::v7400::AttributeValue;
use fbxcel_dom::v7400::object::property::PropertiesHandle;
use fbxcel_dom::v7400::Document;

#[allow(unused)]
pub fn verify(doc: &Document) -> Result<Vec<String>, anyhow::Error> {
    let node = doc
        .tree()
        .root()
        .children_by_name("Creator")
        .next()
        .ok_or_else(|| format_err!("FBXHeaderExtension not found in file."))?;

    let creator = &node.attributes()[0];
    if let AttributeValue::String(c) = creator {
        if c.contains("Blender") {
            let axis = get_coordinate_axis(doc)
                .ok_or_else(|| format_err!("Could not find coordinate axis."))?;

            let correct = CoordinateAxis {
                up: [0, 0, 1].into(),
                front: [0, 1, 0].into(),
                coord: [-1, 0, 0].into(),
            };

            if axis != correct {
                return Ok(vec![format!(
                    "File has incorrect Blender Coordinate Axis. Expected: [{:?}] actual [{:?}]",
                    correct, axis
                )]);
            }
        }
    }

    Ok(vec![])
}

#[derive(Debug, PartialEq)]
struct CoordinateAxis {
    up: Vector3<i8>,
    front: Vector3<i8>,
    coord: Vector3<i8>,
}

fn get_coordinate_axis(doc: &Document) -> Option<CoordinateAxis> {
    let global_settings = doc
        .global_settings()
        .ok_or("Count not find global settings in file.")
        .ok()?;

    let up_axis = get_axis(&global_settings, "UpAxis")?;
    let front_axis = get_axis(&global_settings, "FrontAxis")?;
    let coord_axis = get_axis(&global_settings, "CoordAxis")?;

    Some(CoordinateAxis {
        up: up_axis,
        front: front_axis,
        coord: coord_axis,
    })
}

fn get_axis(global_settings: &PropertiesHandle, name: &str) -> Option<Vector3<i8>> {
    let axis =
        if let AttributeValue::I32(v) = global_settings.get_property(name)?.value_part().get(0)? {
            v
        } else {
            return None;
        };

    let sign = if let AttributeValue::I32(v) = global_settings
        .get_property(&(name.to_owned() + "Sign"))?
        .value_part()
        .get(0)?
    {
        v
    } else {
        return None;
    };

    Some(match axis {
        0 => [*sign as i8, 0, 0].into(),
        1 => [0, *sign as i8, 0].into(),
        2 => [0, 0, *sign as i8].into(),
        _ => return None,
    })
}
