use erupt::vk1_0::*;

// macro_rules! descriptors {
//     ($out:ty $(, $member:ident)*) => (
//         #[allow(unsafe_code)]
//         impl $out {
//             #[inline(always)]
//             fn member(name: &str) -> usize {
//                 let mut foo = 0;
//                 $(
//                     if name == stringify!($member) {
//                         let dummy = <$out>::default();
//                         let dummy_ptr = (&dummy) as *const _;
//                         let member_ptr = (&dummy.$member) as *const _;

//                         let offset = member_ptr as usize - dummy_ptr as usize;
//                         return offset;
//                     }
//                 )*

//                 foo
//             }
//         }
//     )
// }
// #[test]
// fn test_descriptor() {
//     println!("Foobar! {}", VertexPosition::member("position"));
//     println!("VertexPos2Color! {}", VertexPos2Color::member(""));
// }

// macro_rules! descriptors {
//     ($struct_name:expr) => {
//         pub fn get_binding_desc<'a>(binding: u32) -> VertexInputBindingDescriptionBuilder<'a> {
//             VertexInputBindingDescriptionBuilder::new()
//                 .binding(binding)
//                 .stride(std::mem::size_of::<Self>() as u32)
//                 .input_rate(VertexInputRate::VERTEX)
//         }

//         pub fn get_attribute_desc<'a>(
//             binding: u32,
//         ) -> Vec<VertexInputAttributeDescriptionBuilder<'a>> {
//             vec![VertexInputAttributeDescriptionBuilder::new()
//                 .binding(binding)
//                 .location(0)
//                 .format(Format::R32G32B32_SFLOAT)
//                 .offset(0)]
//         }
//     };
// }
#[derive(Default, Debug, Clone)]
pub struct VertexPosition {
    pub position: [f32; 3],
}
// descriptors!(VertexPosition, position);

impl VertexPosition {
    pub fn get_binding_desc<'a>(binding: u32) -> VertexInputBindingDescriptionBuilder<'a> {
        VertexInputBindingDescriptionBuilder::new()
            .binding(binding)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(VertexInputRate::VERTEX)
    }

    pub fn get_attribute_desc<'a>(binding: u32) -> Vec<VertexInputAttributeDescriptionBuilder<'a>> {
        vec![VertexInputAttributeDescriptionBuilder::new()
            .binding(binding)
            .location(0)
            .format(Format::R32G32B32_SFLOAT)
            .offset(0)]
    }
}
// vulkano::impl_vertex!(VertexPosition, position);

#[derive(Default, Debug, Clone)]
pub struct VertexPos2Color {
    pub position: [f32; 2],
    pub color: [f32; 3],
}
// descriptors!(VertexPos2Color, position, color);

// vulkano::impl_vertex!(VertexPos2Color, position, color);

#[derive(Default, Debug, Clone)]
pub struct VertexNormal {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
// vulkano::impl_vertex!(VertexNormal, position, normal);

#[derive(Default, Debug, Clone)]
pub struct VertexNormalTangent {
    position: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 4],
}
// vulkano::impl_vertex!(VertexNormalTangent, position, normal, tangent);

#[derive(Default, Debug, Clone)]
pub struct VertexPBR {
    position: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 4],
    tex_coord0: [f32; 2],
}
// vulkano::impl_vertex!(VertexPBR, position, normal, tangent, tex_coord0);
