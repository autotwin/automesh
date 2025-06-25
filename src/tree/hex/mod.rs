pub mod edge_template_1;
pub mod edge_template_2;
pub mod edge_template_3;
pub mod edge_template_4;
pub mod face_template_0;
pub mod face_template_1;
pub mod vertex_template_1; // (O, A, AB, B) | (o, a, ab, b)
pub mod vertex_template_2; // (O, a, ab, b) | (O, a, ab, b) has smaller (ab) and c(ab)
pub mod vertex_template_3; // (O, A, AB, B) | (o, A, AB, b)
pub mod vertex_template_4; // (O, A, AB, B) | (o, A, AB, B)
pub mod vertex_template_5; // (O, A, AB, B) | (o, A, ab, b)
pub mod vertex_template_6; // (O, A, AB, b) | (o, A, ab, b) has smaller c(b) but should not
pub mod vertex_template_7; // (O, a, ab, b) | (o, a, ab, b) can have smaller c(ab)
pub mod vertex_template_8; // (O, a, AB, B) | (o, a, ab, b)
