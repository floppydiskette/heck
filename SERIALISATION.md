# how map files are saved
## .map file format (uncompiled maps)
idk i'm just planning on using toml or something with serde

## .h2m file format (compiled maps)
```
<H2MAP> (magic, 5 bytes)
<version> (u8)
<map name> (u8 length, followed by string)
<terrain> (u8 length, followed by string; if length is 0, there is no terrain)
<skybox> (u8 length, followed by string; if length is 0, there is no skybox)
<amount of nodes> (u32)
<node data> (see below for definition)
```

### node data
#### node data header
```
<length of this node's data> (u32)
<node type> (u16)
```
#### entity node data
```
<entity id> (u32)
<entity name> (u16 length, followed by string)
<entity children> (see below for definition)
<entity parent> (u32 of parent's id, 0 if no parent)
<count of entity components> (u32)
<entity component data> (see below for definition)
```
##### entity children
```
<count of entity children> (u32)
<entity child 1 id> (u32)
<entity child 2 id> (u32)
...
```
##### entity component data
```
<component type> (u32, id of the component type)
<component parameter 1 name> (u16 length, followed by string)
<component parameter 1 value> (u32 length, data varies between types)
<component parameter 2 name> (u16 length, followed by string)
<component parameter 2 value> (u32 length, data varies between types)
...
```
#### component node data
```
<component id> (u32)
<component name> (u16 length, followed by string)
<component parameters> (see below for definition)
```
##### component parameters
```
<count of component parameters> (u32)
<component parameter 1 name> (u16 length, followed by string)
<component parameter 1 value> (u32 length, data varies between types)
etc...
```

### component parameter types
#### string (mesh name, texture name, etc)
```
<length of string> (u32)
<string data>
```
#### float
```
<f32>
```
#### bool
```
<u8>
```
#### vec3
```
<f32>
<f32>
<f32>
```
#### vec4/quaternion
```
<f32>
<f32>
<f32>
<f32>
```