# FBX Sanitizer
FBX Sanitizer is a single-exe CLI application to check FBX files for common issues such as scaling, rotation, and triangulation. Running this application on an FBX file verifies that the asset will import into Unity with unit scale and zero rotation.

FBX Santizer does not depend on the FBX SDK, and is a single static executable. It is lightweight, incredibly fast, and can analyze hundreds of FBX files per second. FBX parsing is provided by the wonderful [fbxcel-dom](https://github.com/lo48576/fbxcel). The tool is designed to analyze exports from Blender, Maya, and 3DS Max, but should also support other programs.

Supported platforms are Windows, Linux, and OSX.

It is recommended to use this with the [Pontoco Unity Import/Export settings](https://www.notion.so/Pontoco-FBX-Import-Export-Settings-7b363beb7f7249f2bca6d6bdbcae6c5d), which provides export presets for all of the above modeling programs, and a Unity import preset. 

## Checks
See `checks/` for a detailed list of checks and reasonings. In summary:
 - **Identity Transform**: Verifies a single root object has an identity transform.
 - **Correct Coordinate Axis**: Verifies the file is saved with a coordinate axis that will result in a zero rotation. This is unique for each export program.
 - **Units In Meters**: Verifies the file is in Meters units. ([ignored for Maya exports](http://uninomicon.com/fbx_importing#folded_1929534150840c69a4accdff7b5c8f75_2))
 - **No Quads**: Verifies there are no quads or ngons. Unity's will not triangulate a mesh in the same way that Substance Painter will.
 - **Contains Normals**: Verifies all meshes contain normals. Unity's 'calculate normals' is not great -- it's much better to use your modeling program.
 - **Is Binary**: Verifies the file is saved in the FBX Binary format. (Blender can't open ASCII files)
 - **Bounding Box**: Verifies any given mesh is not massive or tiny. This can cause "Generate Lightmap UVs" in Unity to fail.
 - **No Scale Compensation**: Maya animations use Scale Compensation by default. Unity (or any other tool) doesn't support this attribute.
 - **No Namespaces**: Requires all model names to be exported without namespaces.

## Usage
```
USAGE:
    fbx_sanitizer.exe [FLAGS] <files>...

FLAGS:
        --dump-structure    Writes a loosely yaml-structured version of the binary file to <file>_structure.yml.
                            Suitable for debugging and inspection.
    -h, --help              Prints help information
        --summary           Outputs a one-line summary for each fbx file passed in, rather than all errors.
    -V, --version           Prints version information

ARGS:
    <files>...    A set of fbx files to analyze.
```

## Installation
1. Install [Rust](https://www.rust-lang.org/tools/install). 
2. Navigate to the project folder and run `cargo build --release`
3. A standalone executable will be generated in the `target/release` folder for your platform.

## Contribution
Contributions are welcome!

## License
FBX Sanitizer is licensed under the MIT license. But please consider contributing back up-stream if you make tweaks.

## Additional Reading
- http://uninomicon.com/fbx_importing
- https://code.blender.org/2013/08/fbx-binary-file-format-specification/
