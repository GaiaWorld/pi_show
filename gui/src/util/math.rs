use cg::{Quaternion, Vector3, Matrix4};
/**
 * Decomposes the current Matrix into a translation, rotation and scaling components
 * @param scale defines the scale vector3 given as a reference to update
 * @param rotation defines the rotation quaternion given as a reference to update
 * @param translation defines the translation vector3 given as a reference to update
 * @returns true if operation was successful
 */
pub fn decompose(mat4: &Matrix4<f32>, scale: Option<&mut Vector3<f32>>, rotation: Option<&mut Quaternion<f32>>, translation: Option<&mut Vector3<f32>>) -> bool {
    let m: &[f32; 16] = mat4.as_ref();
    match translation {
        Some(t) => {
            t.x = m[12];
            t.y = m[13];
            t.z = m[14];
        },
        None => (),
    }
    
    let ss = Vector3::new(0.0, 0.0, 0.0);
    let scale = match scale {
        Some(s) => {
            s.x = f32::sqrt(m[0] * m[0] + m[1] * m[1] + m[2] * m[2]);
            s.y = f32::sqrt(m[4] * m[4] + m[5] * m[5] + m[6] * m[6]);
            s.z = f32::sqrt(m[8] * m[8] + m[9] * m[9] + m[10] * m[10]);

            if determinant(mat4) <= 0.0 {
                s.y = s.y * -1.0;
            }
            s
        },
        None => &ss,
    };

    if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
        match rotation {
            Some(r) => {
                r.v.x = 0.0;
                r.v.y = 0.0;
                r.v.z = 0.0;
                r.s = 1.0;
            },
            None => (),
        }
        return false;
    }

    match rotation {
        Some(r) => {
            let (sx, sy, sz) = (1.0 / scale.x, 1.0 / scale.y, 1.0 / scale.z);
            let ma = Matrix4::new(
                m[0] * sx, m[1] * sx,  m[2] * sx, 0.0,
                m[4] * sy, m[5] * sy,  m[6] * sy, 0.0,
                m[8] * sz, m[9] * sz, m[10] * sz, 0.0,
                0.0,       0.0,        0.0, 1.0
            );
            from_rotation_matrix_to_rotation(&ma, r);
        },
        None => (),
    }
    true
}

/**
 * Gets the determinant of the matrix
 * @returns the matrix determinant
 */
pub fn determinant(m: &Matrix4<f32>) -> f32 {
    let m: &[f32; 16] = m.as_ref();

    let temp1 = m[10] * m[15] - m[11] * m[14];
    let temp2 = m[9] * m[15] - m[11] * m[13];
    let temp3 = m[9] * m[14] - m[10] * m[13];
    let temp4 = m[8] * m[15] - m[11] * m[12];
    let temp5 = m[8] * m[14] - m[10] * m[12];
    let temp6 = m[8] * m[13] - m[9] * m[12];

    return 
        m[0] * (m[5] * temp1 - m[6] * temp2 + m[7] * temp3) -
        m[1] * (m[4] * temp1 - m[6] * temp4 + m[7] * temp5) +
        m[2] * (m[4] * temp2 - m[5] * temp4 + m[7] * temp6) -
        m[3] * (m[4] * temp3 - m[5] * temp5 + m[6] * temp6)
    ;
}

/**
 * Updates the given quaternion with the given rotation matrix values
 * @param matrix defines the source matrix
 * @param result defines the target quaternion
 */
pub fn from_rotation_matrix_to_rotation(m: &Matrix4<f32>, result: &mut Quaternion<f32>) {
    let data: &[f32; 16] = m.as_ref();

    let (m11, m12, m13) = (data[0], data[4], data[8]);
    let (m21, m22, m23) = (data[1], data[5], data[9]);
    let (m31, m32, m33) = (data[2], data[6], data[10]);
    let trace = m11 + m22 + m33;
    let s;

    if trace > 0.0 {

        s = 0.5 / f32::sqrt(trace + 1.0);

        result.s = 0.25 / s;
        result.v.x = (m32 - m23) * s;
        result.v.y = (m13 - m31) * s;
        result.v.z = (m21 - m12) * s;
    } else if m11 > m22 && m11 > m33 {

        s = 2.0 * f32::sqrt(1.0 + m11 - m22 - m33);

        result.s = (m32 - m23) / s;
        result.v.x = 0.25 * s;
        result.v.y = (m12 + m21) / s;
        result.v.z = (m13 + m31) / s;
    } else if m22 > m33 {

        s = 2.0 * f32::sqrt(1.0 + m22 - m11 - m33);

        result.s = (m13 - m31) / s;
        result.v.x = (m12 + m21) / s;
        result.v.y = 0.25 * s;
        result.v.z = (m23 + m32) / s;
    } else {

        s = 2.0 * f32::sqrt(1.0 + m33 - m11 - m22);

        result.s = (m21 - m12) / s;
        result.v.x = (m13 + m31) / s;
        result.v.y = (m23 + m32) / s;
        result.v.z = 0.25 * s;
    }
}