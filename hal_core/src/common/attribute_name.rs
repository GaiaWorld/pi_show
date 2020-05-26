/**
 * Attribute的名字，类型可以更改，
 * 注：请尽量使用内置的Attribute名，以便于内部加速
 */
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum AttributeName {
    Position,       // shader attribute：position，一般是vec3
    Normal,         // shader attribute：normal，一般是vec3
    Color,          // shader attribute：color，一般是vec4
    UV0,            // shader attribute：uv0，一般是vec2
    UV1,            // shader attribute：uv1，一般是vec2
    SkinIndex,      // shader attribute：skinIndex，一般是vec4
    SkinWeight,     // shader attribute：skinWeight，一般是vec4
    Tangent,        // shader attribute：tangent，一般是vec3
    BiNormal,       // shader attribute：binormal，一般是vec3
    UV2,            // shader attribute：uv2，一般是vec2
    UV3,            // shader attribute：uv3，一般是vec2
    UV4,            // shader attribute：uv4，一般是vec2
    UV5,            // shader attribute：uv5，一般是vec2
    UV6,            // shader attribute：uv6，一般是vec2
    UV7,            // shader attribute：uv7，一般是vec2
    UV8,            // shader attribute：uv8，一般是vec2
    Custom(String), // 自定义名字，无非必要，最好不用,
}

impl From<&str> for AttributeName {
    fn from(name: &str) -> AttributeName {
        match name {
            "position" => AttributeName::Position,
            "normal" => AttributeName::Normal,
            "color" => AttributeName::Color,
            "uv0" => AttributeName::UV0,
            "uv1" => AttributeName::UV1,
            "skinIndex" => AttributeName::SkinIndex,
            "skinWeight" => AttributeName::SkinWeight,
            "tangent" => AttributeName::Tangent,
            "binormal" => AttributeName::BiNormal,
            "uv2" => AttributeName::UV2,
            "uv3" => AttributeName::UV3,
            "uv4" => AttributeName::UV4,
            "uv5" => AttributeName::UV5,
            "uv6" => AttributeName::UV6,
            "uv7" => AttributeName::UV7,
            "uv8" => AttributeName::UV8,
            n @ _ => AttributeName::Custom(n.to_string()),
        }
    }
}

impl Into<String> for AttributeName {
    fn into(self) -> String {
        match self {
            AttributeName::Position => "position".to_string(),
            AttributeName::Normal => "normal".to_string(),
            AttributeName::Color => "color".to_string(),
            AttributeName::UV0 => "uv0".to_string(),
            AttributeName::UV1 => "uv1".to_string(),
            AttributeName::SkinIndex => "skinIndex".to_string(),
            AttributeName::SkinWeight => "skinWeight".to_string(),
            AttributeName::Tangent => "tangent".to_string(),
            AttributeName::BiNormal => "binormal".to_string(),
            AttributeName::UV2 => "uv2".to_string(),
            AttributeName::UV3 => "uv3".to_string(),
            AttributeName::UV4 => "uv4".to_string(),
            AttributeName::UV5 => "uv5".to_string(),
            AttributeName::UV6 => "uv6".to_string(),
            AttributeName::UV7 => "uv7".to_string(),
            AttributeName::UV8 => "uv8".to_string(),
            AttributeName::Custom(n) => n.clone(),
        }
    }
}

impl AttributeName {
    /**
     * 内置Attribute名字的就是上面的16个
     */
    pub fn get_builtin_count() -> u32 {
        16
    }
}
