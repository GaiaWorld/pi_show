
extern crate wasm_bindgen;
extern crate bincode;
extern crate fx_hashmap;
extern crate gui;
#[macro_use]
extern crate serde;

use std::mem::transmute;

use js_sys::Uint8Array;

use fx_hashmap::FxHashMap32;
use hash::XHashMap;
use gui::single::{style_parse::parse_class_map_from_string};
use gui::{single::Class, font::font_cfg::{FontCfg, GlyphInfo, MetricsInfo, CharSdf}};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize, Debug)]
pub struct Result {
	pub err: Option<String>,
	pub bin: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlyphInfo1 {
	unicode: u32,
	glyph: GlyphInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontAtlasData {
    name: String,
    metrics: MetricsInfo,
    glyphs: Vec<GlyphInfo1>,
	// atlas: Vec<CharSdf>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct FontAtlasData1 {
//     cfg: FontCfg,
// 	atlas: Vec<CharSdf>,
// }

/**
 * 在指定上下文中创建一个 文本样式表
 * __jsObj: class样式的文本描述
 */
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn serialize_class_map(value: &str) -> JsValue {
    let r = match parse_class_map_from_string(value) {
        Ok(r) => match bincode::serialize(&r) {
            Ok(bin) => Result{err: None, bin: Some(bin)},
            Err(r) => Result{err: Some(r.to_string()), bin: None},
        },
        Err(r) => Result{err: Some(r), bin: None}
    };

	JsValue::from_serde(&r).unwrap()
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn deserialize_class_map(bin: &[u8]) {
    let r: FxHashMap32<usize, Class> = match bincode::deserialize(bin) {
        Ok(r) => r,
        Err(e) => {
            println!("deserialize_class_map error: {:?}", e);
            return;
        }
    };
    // println!("r: {:?}", r);
}

// 序列化sdf配置的json格式的文件
#[wasm_bindgen]
pub fn serialize_sdf_json(s: &str) -> Uint8Array {
	let info: FontAtlasData = match serde_json::from_str(s) {
		Ok(r) => r,
		Err(e) => {
			log::error!("serialize_sdf_json fail, {:?}", e);
			return Uint8Array::from(Vec::new().as_slice());
		}
	};
	let mut map = XHashMap::default();
	for item in info.glyphs.into_iter() {
		map.insert(unsafe{transmute::<u32, char>(item.unicode)}, item.glyph);
	}

	// let r = FontAtlasData1 {
	// 	cfg: FontCfg {
	// 		name: info.name,
	// 		metrics: info.metrics,
	// 		glyphs: map,
	// 	},
	// 	atlas: info.atlas,
	// };
	let r = FontCfg {
		name: info.name,
		metrics: info.metrics,
		glyphs: map,
	};

	match bincode::serialize(&r) {
        Ok(r) => Uint8Array::from(r.as_slice()),
        Err(e) => {
            log::error!("serialize_sdf_json error: {:?}", e);
            return Uint8Array::from(Vec::new().as_slice());
        }
    }
}

#[wasm_bindgen]
pub fn init_log() {
    pi_web_logger::init_with_level(pi_web_logger::Level::Info);
	log::info!("init_logger ok!");
}


#[test]
fn tests() {
	let r = "
.t_s_12181a_3{
	text-shadow: 0px 3px 3px #12181a;
}
.t_s_bb7be5_3{
	text-shadow: 0px 3px 3px #bb7be5;
}
.t_s_7031e4_3{
	text-shadow: 0px 3px 3px #7031e4;
}";
let r = parse_class_map_from_string(r);
println!("===================={:?}", r);
}

#[test]
fn test1() {
	let r = ".564697228{
		color: #964f29ff;
		font-size: 48px;
	}.4038825699{
		position: relative;
		width: auto;
		align-items: flex-end;
		align-content: flex-end;
	}.2979274224{
		position: absolute;
		width: 100%;
		height: 100%;
		left: 0;
		top: 0;
	}
	
	.4245119708{
		position: relative;
		justify-content: center;
		align-items: center;
		align-content: center;
		text-align: center;
		width: 100%;
		height: 100%;
	}.1922053647{
		position: absolute;
		width: 100%;
		height: 100%;
	}
	.1163764969{
		width: 100%;
		height: 100%;
	}
	
	.2382074929{
		position: absolute;
		right: 6%;
		pointer-events: none;
	}
	.4065118656{
		position: absolute;
		width: 300px;
		justify-content: center;
		left: 50%;
		transform: translateX(-50%);
		pointer-events: none;
	}@keyframes arrowAnima {
		0% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
		100% {
			opacity: 1;
		}
	}
	@keyframes nomal {
		0% {
			opacity: 1;
		}
		100% {
			opacity: 1;
		}
	}.290743885{
		width: 100%;
		height: 100%;
		position: absolute;
	}@keyframes coverBg {
		0% {
			opacity: 0;
		}
		100% {
			opacity: 1;
		}
	}
	@keyframes nomal {
		0% {
			opacity: 1;
		}
		100% {
			opacity: 1;
		}
	}.1223689111{
		text-shadow: 2px 4px 2px #9b5500;
	}
	
	.109831728{
		width: 100%;
		text-align: center;
		position: relative;
	}.1784099349{
		position: absolute;
		width: 100%;
		height: 100%;
		top:0;
		right: 0;
	}
	
	.4134732493{
		display: flex;
		justify-content: center;
		align-items: center;
		align-content: center;
		position: relative;
		left:0;
		top:0;
		z-index: 2;
		width:100%;
		height: 100%;
	}.4169142760{
	  width:60px;
	  height:60px;
	  margin-top: -14px;
	}
	.1845872682{
		width: auto;
		height: auto;
	}
	
	.2305303851{
		position: absolute;
		margin-left: -25px;
		margin-top: 10px;
		justify-content: center;
		align-items: center;
		z-index: 1;
	}
	.834487629{
	  position: relative;
	  top: 50%;
	  transform: translateY(-50%);
	}
	.3405721134{
	  position: relative;
	  top: 50%;
	  transform: scaleX(-1) translateY(-50%);
	}
	.1809269170{
	  height: auto;
	  position: relative;
	}
	.1810768609{
	  position: absolute;
	  width: 100%;
	  height: 100%;
	  justify-content: center;
	  align-items: center;
	  align-content: center;
	}
	.3122116307{
			justify-content: center;
			position: absolute;
			left:50%;
			transform: translateX(-50%);
			width: 240px;
			height: 36px;
			z-index: 1;
		}
		.583069241{
			position: absolute;
			top:-2px;
		}
	.3001878544{
			width:100%;
			height:100%;
			overflow: hidden;
		}
	.1219296203{
		position: relative;
		width: auto;
	}
	.3926229798{
	  position: absolute;
	  width: 50%;
	  height: 100%;
	  left: 0px;
	}
	.1673599174{
	  position: absolute;
	  width: 50%;
	  height: 100%;
	  right: 0px;
	  transform: scale(-1,-1);
	}
	.3022569657{
	  position: absolute;
	  width: 100%;
	  height: 100%;
	  transition: all 1s ease;
	}
	.3404892167{
	  position: absolute;
	  width: 100%;
	  flex-wrap: nowrap;
	}
	.3965519869{
	  height: 100%;
	  flex-wrap: nowrap;
	  width: 100%;
	  justify-content: center;
	  position: absolute;
	}
	.493887116{
	  text-stroke: 1px #434343;
	}
	.2920522154{
		position: absolute;
		width: 100%;
		height: 100%;
		justify-content: center;
		align-items: center;
		align-content: center;
	}
	.2814775262{
		left: 50%;
		top: 50%;
		transform: translate(-50%,-50%);
	}
	.823897201{
		position: absolute;
		width: 100%;
		height: 30px;
		text-align: center;
	}
	
	.2485527760{ font-size: 44px; color: #d5475a; font-family:1250225397; font-weight:600;  }
	.1408246497{ font-size: 44px; color: #64615d; font-family:1250225397; font-weight:600;  }
	.1485761207{ font-size: 44px; color: #51965a; font-family:1250225397; font-weight:600;  }
	.1252867292{ font-size: 44px; color: #78624a; font-family:1250225397; font-weight:600;  }
	.3635411103{ font-size: 42px; color: #884233; font-family:1250225397; font-weight:600;  }
	.3005773117{ font-size: 38px; color: #4e3e29; font-family:1250225397; font-weight:600; }
	.2400432589{ font-size: 40px; color: #a08350; font-family:3448399207; }
	.1801139440{ font-size: 40px; color: #78624a; font-family:3448399207; }
	.1476882257{ font-size: 32px; color: #3b2e20; font-family:1250225397; font-weight:600;  }
	.2685314177{ font-size: 40px; color: #78624a; font-family:3448399207}
	.3926567133{ font-size: 32px; color: #f9f4dd; font-family:3448399207}
	.1667315598{ font-size: 48px; color: #c4e3f6; font-family:3448399207}
	.3961503899{ font-size: 42px; color: #c4e3f6; font-family:3448399207}
	.3662561325{ font-size: 42px; color: #dadada; font-family:3448399207}
	.2118405139{ font-size: 48px; color: #dadada; font-family:3448399207}
	.4134881012{ font-size: 36px; color: #c4e3f6; font-family:3448399207}
	.2034074251{ font-size: 36px; color: #dadada; font-family:3448399207}
	.3229473327{ font-size: 48px; color: #f1cfcf; font-family:3448399207}
	
	.3220951426{ font-size: 36px; color: #fffc8e; font-family:3448399207;text-stroke: 2px #ca6f3c }
	.3788084635{ font-size: 36px; color: #4e3e29; font-family:1250225397; font-weight:600;  }
	.3894216077{ font-size: 32px; color: #413e36; font-family:1250225397; font-weight:600;  }
	.3571294696{ font-size: 30px;color: #413e36;font-family:1250225397; font-weight:600;  }
	.1725536768{ font-size: 50px; color: #78624a; font-family:3448399207}
	.2450669497{ font-size: 48px; color: #78624a; font-family:3448399207}
	.3851556030{ font-size: 34px; color: #be9d78; font-family:3448399207}
	.1673968075{ font-size: 38px; color: #78624a; font-family:3448399207}
	.1645318204{ font-size: 38px; color: #78624a; font-family:1250225397}
	.1944096801{ font-size: 34px; color: #e9cb7a; font-family:1250225397; font-weight:600;  }
	.1973595934{ font-size: 36px; color: #4d7b9f; font-family:1250225397; font-weight: bold }
	.660871251{ font-size: 36px; color: #a64141; font-family:1250225397; font-weight: bold }
	.2922908533{ font-size: 36px; color: #676e48; font-family:1250225397; font-weight: bold }
	.3184516841{ font-size: 32px; color: #7b654c; font-family:1250225397; }
	.2227012729{ font-size: 36px; color: #70584a; font-family:1250225397; font-weight: bold }
	.3879632713{ font-size: 34px; color: #676e48; font-family:1250225397; font-weight: bold }
	.1149676068{ font-size: 40px; color: #70584a; font-family:1250225397; font-weight: bold }
	.1085680510{ font-size: 30px; color: #e7e3d3; font-family:1250225397; font-weight: bold }
	.3368697891{ font-size: 30px; color: #c2c1c0; font-family:1250225397; font-weight: bold }
	.1497016812{ font-size: 30px; color: #a1ff8e; font-family:1250225397; font-weight: bold;text-stroke: 2px #3a4439; }
	.1694498405{ font-size: 32px; color: #70584a; font-family:1250225397; font-weight: bold }
	.1557765401{font-size: 32px;font-family:1250225397; color: #51965a;}
	.1869695810{font-size: 24px;font-family:1250225397; font-weight:bold; color: #78624a;}
	.2318613219{ font-size: 28px; color: #49aa54; font-family:1250225397; }
	.22343964{ font-size: 28px; color: #78624a; font-family:1250225397; }
	.2692835489{ font-size: 36px; color: #379b3b; font-family:1250225397; font-weight:bold; }
	.1489116840{ font-size: 34px; color: #4d7b9f; font-family:1250225397; }
	.481052542{ font-size: 32px; color: #fbfff3; font-family:1250225397; }
	.3409095770{ font-size: 40px; color: #70584a; font-family:1250225397;}
	.2388207383{ font-size: 34px; color: #70584a; font-family:1250225397;}
	.2705244657{ font-size: 34px; color: #70584a; font-family:1250225397;}
	.1555605003{font-size: 38px; color: #437f52; font-family:1250225397;}
	.3237464925{font-size: 46px; color: #78624a; font-family:1250225397; font-weight:bold;}
	.2521207090{font-size: 46px; color: #a64141; font-family:1250225397; font-weight:bold;}
	.277670158{font-size: 46px; color: #4f824d; font-family:1250225397; font-weight:bold;}
	.1090276832{font-size: 46px; color: #4f824d; font-family:1250225397;}
	.1260380291{font-size: 32px; color: #78624a; font-family:1250225397; font-weight:bold;}
	.4139580032{font-size: 32px; color: #884233; font-family:1250225397; font-weight:bold;}
	.597805854{font-size: 32px; color: #884233; font-family:1250225397;}
	.1431378332{font-size: 32px; color: #1FC676; font-family:1250225397;}
	.732582233{font-size: 26px; color: #1FC676; font-family:1250225397;}
	.1652345519{font-size: 32px; color: #78624a; font-family:1250225397;}
	.3064334178{font-size: 32px; color: #4a8057; font-family:1250225397;}
	.4006649067{font-size: 36px; color: #78624a; font-family:3448399207;}
	.2386210628{font-size: 36px; color: #86783f; font-family:1250225397;}
	.287565294{font-size: 36px; color: #884233; font-family:1250225397;}
	.1177521519{font-size: 36px; color: #78624a; font-family:1250225397;}
	.1344115669{font-size: 25px; color: #5f532a; font-family:1250225397;font-weight: bold;}
	.2494229312{font-size: 23px; color: #60542a; font-family:1250225397;}
	.2167127308{font-size: 26px;color: #37e767;font-family:1250225397;}
	.4111238940{font-size: 26px;color: #ff3434;font-family:1250225397;}
	.3284695006{font-size: 40px;color: #78624a;font-family:1250225397;}
	.2266622472{font-size: 40px;color: #b93832;font-family:1250225397;}
	.1417692445{font-size: 40px;color: #4a8057;font-family:1250225397;}
	.2713913784{font-size: 32px;color: #4a8057;font-family:1250225397;}
	.2919368502{font-size: 40px;color: #884233;font-family:1250225397;}
	.1775614586{font-size: 32px;color: #884233;font-family:1250225397;}
	.1305873268{font-size: 36px;color: #d4dfef;font-family:1250225397;line-height: 100px;}
	.2903481752{font-size: 36px;color: #fe3636;font-family:1250225397;line-height: 100px;}
	.1825197402{font-size: 36px;color: #e1ca86;font-family:1250225397;}
	.2536386139{font-size: 26px;color: #ffffff;font-family:1250225397;text-stroke: 2px #4d3917}
	.4150086299{font-size: 26px;color: #ffffff;font-family:1250225397;}
	.3689582080{font-size: 32px;color: #b6b6b6;font-family:1250225397;}
	.3663931595{color: #fff;font-size: 36px;font-family:1250225397;}
	.2628378856{color: #fff;font-size: 26px;font-family:1250225397;}
	.3122736525{color: #38de47;font-size: 32px;font-family:1250225397;}
	.2906176375{color: #37e767;font-size: 36px;font-family:1250225397;}
	
	.2020526003{font-size: 36px; color: #af6234; font-family:3448399207;font-weight:bold;}
	.1566705878{ font-size: 36px; color: #64615d; font-family:1250225397; font-weight:600; }
	.2326707511{ font-size: 42px; color: #78624a; font-family:1250225397; font-weight:600; }
	.2230660371{ font-size: 38px; color: #78624a; font-family:1250225397; font-weight:600;line-height: 60px; }
	.673166149{ font-size: 38px; color: #78624a; font-family:1250225397; font-weight:600; }
	.3322381050{ font-size: 38px; color: #379b3b; font-family:1250225397; font-weight:600; }
	.2660331417{ font-size: 32px; color: #f2e3dc; font-family:1250225397; font-weight:600;  }
	.1197035794{ font-size: 32px; color: #f1eeda; font-family:1250225397; font-weight:600;  }
	.820350947{ font-size: 32px; color: #d5e1ea; font-family:1250225397; font-weight:600;  }
	.1077437076{ font-size: 32px; color: #d5ead7; font-family:1250225397; font-weight:600;  }
	.4142134030{ font-size: 38px; color: #70584a; font-family:1250225397; font-weight: bold }
	.1519523564{ font-size: 38px; color: #70584a; font-family:3448399207; }
	.751475113{ font-size: 52px; color: #78624a; font-family:3448399207}
	.1296286758{ font-size: 36px; color: #e9cb7a; font-family:1250225397; font-weight:600; }
	.1362788849{ font-size: 34px; color: #ffffff; font-family:1250225397; font-weight:600; }
	.796596172{ font-size: 34px; color: #c4e3f6; font-family:1250225397; font-weight:600; }
	.3257267835{ font-size: 36px; color: #49aa54; font-family:1250225397; font-weight:600; }
	.3840764139{ font-size: 42px; color: #e7dd9c; font-family:3448399207}
	.968980348{ font-size: 32px; color: #e7dd9c; font-family:3448399207; }
	.674773642{ font-size: 36px; color: #78624a; font-family:1250225397; font-weight:600; }
	.1981545819{ font-size: 36px; color: #ffffff; font-family:1250225397; font-weight:600; }
	.1141762450{ font-size: 34px; color: #78624a; font-family:1250225397; font-weight:600; }
	.3399633753{ font-size: 38px;color: #a14000;font-family:3448399207; }
	.1608715464{ font-size: 46px;color: #78624a;font-family:3448399207; }
	.3650261359{ font-size: 32px;color: #ffffff;font-family:1250225397; font-weight:600; }
	.4131221689{ font-size: 44px;color: #f1cfcf;font-family:3448399207; }
	.2981850137{ font-size: 44px;color: #c4e3f6;font-family:3448399207; }
	.58998816{ font-size: 44px;color: #ffffff;font-family:3448399207; }
	.3069585780{ font-size: 44px;color: #faf9d0;font-family:3448399207; }
	.4213055873{font-size: 44px;font-family:3448399207;color: #dadada;}
	.809999668{ font-size: 38px;color: #faf9d0;font-family:3448399207; }
	.1681741326{ font-size: 38px; color: #c4e3f6; font-family:3448399207}
	.2095352132{font-size: 38px;font-family:3448399207;color: #dadada;}
	.2252847507{ font-size: 42px; color: #f6dac4; font-family:3448399207}
	.4211157512{ font-size: 48px; color: #f6dac4; font-family:3448399207}
	.2666849533{ font-size: 36px;color: #ffffff;font-family:3448399207; }
	.2798229910{font-size: 50px;font-family:1250225397;color: #78624a;}
	.450579198{font-size: 60px;font-family:3448399207;color: #78624a;}
	.1886104651{color: #78624a;font-size: 58px;font-family:3448399207;}
	.1060840413{color: #e4decb;font-size: 48px;font-family:3448399207;}
	.3032637792{color: #a1785c;font-size: 45px;font-family:3448399207;}
	.484893819{color: #a1785c;font-size: 40px;font-family:3448399207;}
	.3965853054{color: #546a84;font-size: 45px;font-family:3448399207;}
	.237295345{color: #546a84;font-size: 40px;font-family:3448399207;}
	.1750476540{color: #c4e3f6;font-size: 46px;font-family:3448399207;}
	.2962741548{color: #f1cfcf;font-size: 46px;font-family:3448399207;}
	.452352246{ font-size: 28px; color: #e6e6e6; font-family:3448399207; }
	.1155656398{color: #857a66;font-size: 50px;font-family:3448399207;}
	.3211049978{color: #856947;font-size: 38px;font-family:3448399207;font-weight: bold;}
	.3434635397{color: #856947;font-size: 36px;font-family:3448399207;}
	.1270898656{color: #856947;font-size: 45px;font-family:3448399207;}
	.3633624115{color: #856947;font-size: 42px;font-family:3448399207;}
	.4213644154{color: #fff;font-size: 46px;font-family:3448399207;}
	.2724297907{color: #fff;font-size: 42px;font-family:3448399207;}
	.1030000820{color: #a6a6b6;font-size: 42px;font-family:3448399207;}
	.3029179147{font-size: 40px;font-family:3448399207;color: #fff5e0;text-stroke: 4px #b98320;text-gradient: linear-gradient(0deg, #fff7cf, #f3ba4c 70%);}
	.3235242726{color: #64593f;font-size: 34px;font-family:3448399207;text-stroke: 1px #fff0c0;}
	.1190101139{color: #fffcf0;font-size: 38px;font-family:3448399207;font-weight: bold;text-stroke: 3px #9b591e;text-shadow: 0px 1px 2px #000000;}
	.2396940041{color: #9b4e47;font-size: 46px;font-family:3448399207;}
	.1561627837{color: #c4e3f6;font-size: 44px;font-family:3448399207;}
	.1603857891{color: #f1cfcf;font-size: 44px;font-family:3448399207;}
	.385556273{color: #736358;font-size: 48px;font-family:3448399207;}
	.798434819{color: #fff;font-size: 38px;font-family:3448399207;}
	.635378172{color: #49aa54;font-size: 38px;font-family:3448399207;}
	.1192825584{color: #ffffff;font-size: 34px;font-family:3448399207;}
	.4211780037{color: #4f5870;font-size: 44px;font-family:3448399207;}
	.3487602436{color: #fff2a4;font-size: 34px;font-family:3448399207;}
	.3149067893{color: #714018;font-size: 34px;font-family:3448399207;}
	.3684288834{color: #d88018;font-size: 64px;font-family:3448399207;text-gradient: linear-gradient(0deg, #fcfcf0, #ebcc8b 70%);text-stroke: 3px #d55700;}
	.3884292507{color: #d88018;font-size: 44px;font-family:3448399207;font-weight:600;text-gradient: linear-gradient(0deg, #f8d98b 30%, #fcefc9);text-stroke: 2px #58311d;text-shadow: 0px 0px 3px rgba(0,0,0,0.42); }
	.2016015200{color: #d88018;font-size: 38px;font-family:3448399207;font-weight:600;text-gradient: linear-gradient(0deg, #f8d98b 30%, #fcefc9);text-stroke: 2px #58311d;text-shadow: 0px 0px 3px rgba(0,0,0,0.42); }
	.3507311562{color: #d88018;font-size: 36px;font-family:3448399207;font-weight:600;text-gradient: linear-gradient(0deg, #f8d98b 30%, #fcefc9);text-stroke: 2px #58311d;text-shadow: 0px 0px 3px rgba(0,0,0,0.42); }
	.2985119779{color: #e6dcb3;font-size: 64px;font-family:3448399207;;text-gradient: linear-gradient(0deg, #ffe6ba 30%, #fff8df);text-stroke: 3px #7b624c;}
	.593277524{color: #e6dcb3;font-size: 44px;font-family:3448399207;;text-gradient: linear-gradient(0deg, #ffffff 0%, #ebcc8b);}
	.2227495884{font-family:4149012451; font-size: 80px; text-gradient: linear-gradient(0deg, #f2fdff 0%, #9ba7ff 50%);text-shadow: 0px 0px 5px rgba(0,41,186,1); }
	.1920231932{font-family:4149012451;font-size: 87px;text-shadow: 0px 0px 9px rgba(203,112,80,1);text-gradient: linear-gradient(180deg, #ffd593 30%, #ffffff 100%);}
	.3759588058{font-family:4149012451; font-size: 45px; color: #fff2ce;text-shadow: 0px 0px 5px rgba(255,125,112,0.69); }
	.2449150439{color: #bd461c;font-size: 44px;font-family:3448399207;}
	.1583794249{color: #78624a;font-size: 46px;font-family:3448399207;}
	.1878390282{color: #78624a;font-size: 44px;font-family:3448399207;}
	.1692042151{color: #78624a;font-size: 40px;font-family:3448399207;}
	.1851985880{color: #78624a;font-size: 44px;font-family:3448399207;}
	.15705585{color: #884233;font-size: 50px;font-family:3448399207;}
	.1080533384{color: #8c5507;font-size: 40px;font-family:3448399207;}
	.4274294954{color: #956546;font-size: 28px;font-family:3448399207;font-weight: bold;}
	.2159016410{color: #f2e6cb;font-size: 32px;font-family:3448399207;}
	.3936735656{color: #78624a;font-size: 58px;font-family:3448399207;}
	.2725017245{color: #78624a;font-size: 56px;font-family:3448399207;}
	.2554923433{color: #dbb779;font-size: 46px;font-family:3448399207;}
	.196374607{color: #78624a;font-size: 46px;font-family:3448399207;}
	.2061573807{color: #78624a;font-size: 44px;font-family:3448399207;}
	.2341397637{color: #fff;font-size: 44px;font-family:3448399207;}
	.3896294335{color: #e9e4ae;font-size: 40px;font-family:3448399207;text-stroke:2px #324250;}
	.601337679{color: #78624a;font-size: 48px;font-family:3448399207;}
	.2938962212{color: #78624a;font-size: 50px;font-family:3448399207;}
	.1619068728{color: #c4e3f6;font-size: 34px;font-family:3448399207;}
	.225986704{color: #af6701;font-size: 44px;font-family:3448399207;}
	.8149662{color: #ffffff;font-size: 42px;font-family:3448399207;text-stroke:2px #9e6d4b;text-shadow: 2px 2px 3px rgba(124,73,12,0.5);}
	
	.663883593{color: #858484;font-size: 42px;font-family:3448399207;}
	.14452842{color: #458353;font-size: 42px;font-family:3448399207;}
	.3370513434{color: #2c7b99;font-size: 42px;font-family:3448399207;}
	.2977747916{color: #8250a8;font-size: 42px;font-family:3448399207;}
	.2778950531{color: #bd931c;font-size: 42px;font-family:3448399207;}
	.2449150439{color: #bd461c;font-size: 42px;font-family:3448399207;}
	.3366511622{color: #78624a;font-size: 78px;font-family:3448399207;text-stroke: 2px #a57f3c;text-gradient: linear-gradient(0deg, #ffa92d, #ffe98d 51%, #f6eecb);}
	.1669541078{color: #ffffff;font-size: 32px;font-family:3448399207;}
	.148083616{color: #ffffff;font-size: 25px;font-family:3448399207;}
	.3537104858{color: #70584a;font-size: 45px;font-family:3448399207;}
	.2078703342{color: #af6701;font-size: 44px;font-family:3448399207;}
	.1247970427{color: #ffeab8;font-size: 36px;font-family:1250225397;}
	.947286740{color: #ffeab8;font-size: 44px;font-family:3448399207;text-shadow: 1px 1px 3px #000000;}
	.1247970427{color: #ffeab8;font-size: 36px;font-family:1250225397;}
	.942308785{color: #ffeab8;font-size: 44px;font-family:3448399207;text-shadow: 1px 1px 3px rgba(0,0,0,0.42);}
	.1092760367{font-family:4149012451; font-size: 48px; color: #fffffe; text-shadow: 0px 0px 5px rgba(255,210,112,0.74); }
	
	.2298924266{color: #78624a;font-size: 40px;font-family:1250225397;}
	.2524347860{color: #ffffff;font-size: 32px;font-family:1250225397;text-stroke: 1px #434343;}
	.2900012726{color: #ffffff;font-size: 32px;font-family:1250225397;}
	.2231946226{color: #ffffff;font-size: 38px;font-family:1250225397;}
	.3642116530{color: #ffffff;font-size: 38px;font-family:3448399207;}
	.2990796676{color: #ffffff;font-size: 58px;font-family:3448399207;}
	.1629734219{color: #ffffff;font-size: 56px;font-family:3448399207;}
	.3789083232{color: #ffffff;font-size: 62px;font-family:3448399207;}
	.2011080050{color: #d88018;font-size: 52px;font-family:3448399207;}
	.2674864894{color: #af6701;font-size: 40px;font-family:1250225397;font-weight: bold;}
	.3552134603{color: #78624a;font-size: 40px;font-family:1250225397;font-weight: bold;}
	.2175423639{color: #64615d;font-size: 32px;font-family:1250225397;}
	.2310189262{color: #21881c;font-size: 34px;font-family:1250225397;font-weight: bold;}
	.3078578507{color: #fffefa;font-size: 28px;font-family:1250225397;font-weight: bold;}
	.3219260266{color: #43c448;font-size: 36px;font-family:1250225397;font-weight: bold;}
	.368741790{color: #eadeba;font-size: 30px;font-family:1250225397;}
	.1412255698{color: #fff;font-size: 36px;font-family:1250225397;}
	.191647617{color: #fff;font-size: 36px;font-family:1250225397;line-height: 38px;}
	.1506358762{font-size: 36px;font-family:1250225397;color: #4a8057;}
	.807547538{font-size: 34px;font-family:1250225397;color: #884233;font-weight: bold;}
	.3211851661{font-size: 34px;font-family:1250225397;color: #149195;}
	.108666482{font-size: 40px;font-family:1250225397;color: #78624a;}
	.453384440{font-size: 46px;font-family:1250225397;color: #64615d;font-weight: bold;}
	.2765255649{ font-size: 38px;color: #4f824d;font-family:1250225397; font-weight:600; }
	.3873562433{ font-size: 38px;color: #51965a;font-family:1250225397; font-weight:600; } 
	.3933266722{ font-size: 42px;color: #78624a;font-family:1250225397; font-weight: bold; }
	.2849621036{ font-size: 42px;color: #78624a;font-family:3448399207; }
	.1395214014{ font-size: 32px;color: #ffeab8;font-family:1250225397; font-weight:600; }
	.1926705419{ font-size: 36px;color: #4f824d;font-family:1250225397; font-weight:600; }
	.2400599780{ font-size: 36px;color: #678962;font-family:1250225397; font-weight:600; }
	.2018914270{ font-size: 34px;color: #78624a;font-family:1250225397;}
	.2458588366{ font-size: 36px;color: #51965a;font-family:1250225397; font-weight:600; }
	.1410560921{ font-size: 48px;color: #78624a;font-family:1250225397; font-weight:600; }
	.572079029{ font-size: 46px;color: #78624a;font-family:1250225397; font-weight:600; }
	.3570316991{ font-size: 42px;color: #70584a;font-family:1250225397; font-weight:600; }
	.3966428612{ font-size: 46px;color: #70584a;font-family:3448399207; font-weight:600; }
	.2900113479{ font-size: 58px;color: #78624a;font-family:1250225397; font-weight: bold; }
	.2274527355{ font-size: 46px;color: #78624a;font-family:1250225397; font-weight: bold; }
	.872550005{ font-size: 52px;color: #78624a;font-family:3448399207; }
	.3043108695{ font-size: 40px;color: #504131;font-family:1250225397; font-weight:600;  }
	.84270328{ font-size: 46px;color: #ed437b;font-family:3448399207; }
	.3109567420{ font-size: 46px;color: #946346;font-family:3448399207; }
	.2098974331{ font-size: 40px;color: #78624a;font-family:3448399207; letter-spacing: -3px;}
	.3687462315{font-size: 36px;color: #FFFFFF;font-family:1250225397; font-weight:600; }
	.62401460{font-size: 34px;color: #965151;font-family:1250225397; font-weight:600; }
	.666325573{font-size: 32px;color: #884233;font-family:1250225397; font-weight:600; }
	.1418952937{font-size: 38px;color: #64615d;font-family:1250225397; font-weight:600; }
	.555423566{font-size: 32px;color: #78624a;font-family:1250225397; font-weight:600; }
	.3244128946{font-size: 26px;color: #fff;font-family:1250225397; font-weight:bold; }
	.3151069364{font-size: 34px;color: #64615d;font-family:1250225397; font-weight:600; }
	.2837384201{font-size: 34px;color: #4a8057;font-family:1250225397; font-weight:600; }
	.1222092659{font-size: 34px;color: #4a8057;font-family:1250225397;}
	.3858143360{font-size: 32px;color: #78624a;font-family:1250225397;font-weight: 600;}
	.807547538{font-size: 32px;color: #884233;font-family:1250225397;font-weight:600;}
	.3837267822{font-size: 42px;color: #665d4e;font-family:3448399207;}
	.1737942044{font-size: 40px;color: #70584a;font-family:1250225397;}
	.296771089{ font-family:1250225397;font-size: 38px;color: #af6701;}
	.721554213{font-size: 32px;color: #78624a;font-family:1250225397;}
	.980501490{font-size: 30px;color: #78624a;font-family:1250225397;}
	.1825078227{font-size: 32px;color: #ffffff;font-family:1250225397;}
	.3028991589{font-size: 32px;color: #7bed97;font-family:1250225397;}
	.3763892451{font-size: 40px;color: #b93832;font-family:1250225397;}
	.1505525829{font-size: 34px;color: #6f5c4d;font-family:1250225397;}
	.2184240845{font-size: 40px;color: #319748;font-family:1250225397;font-weight: bold;}
	.2247329825{font-size: 78px;color: #78624a;font-family:1250225397;text-shadow:0px 0px 3px #a57f3c;text-gradient: linear-gradient(0deg, #f6eecb, #ffe98d 51%, #ffa92d)}
	.3127178226{font-size: 38px;color: #70584a;font-family:1250225397; font-weight:600; }
	.1282936511{font-size: 42px;color: #ffeab8;font-family:3448399207;}
	.947286740{font-size: 44px;color: #ffeab8;font-family:3448399207;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);}
	.1563326681{font-size: 44px;color: #ffffff;font-family:3448399207;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);}
	.3266797966{font-size: 34px;color: #fcfee7;font-family:1250225397; font-weight:600; }
	.511174170{font-size: 36px;color: #78624a;font-family:1250225397; font-weight:600; }
	.3294607308{font-size: 48px;color: #78624a;font-family:3448399207;}
	.3738898679{font-size: 48px;color: #78624a;font-family:3448399207; font-weight:600; }
	.2955545823{font-size: 50px;color: #78624a;font-family:3448399207; font-weight:600; }
	.1745627000{font-size: 36px;color: #4e3e29;font-family:1250225397; font-weight:600; }
	.1495685705{font-size: 38px;color: #70584a;font-family:1250225397; font-weight:600; }
	.376184539{ font-size: 34px;color: #51965a;font-family:1250225397; font-weight:600;  }
	.1609848236{ font-size: 34px;color: #965151;font-family:1250225397; font-weight:600;  }
	.1225454606{ font-size: 32px;color: #9b6f23;font-family:1250225397; font-weight:600;  }
	.2134563597{ font-size: 32px;color: #379b3b;font-family:1250225397; font-weight:600;  }
	.4064657905{ font-size: 40px;color: #ffeab8;font-family:3448399207; }
	.2753655042{ font-size: 40px;color: #78624a;font-family:3448399207; }
	.3461013074{ font-size: 38px; color: #807730; font-family:1250225397; font-weight:600;  }
	.699799087{ font-size: 38px; color: #a5732b; font-family:1250225397; font-weight:600;  }
	.3367334139{ font-size: 38px; color: #67808b; font-family:1250225397; font-weight:600;  }
	.4128251613{ font-size: 35px; color: #fce8b8; font-family:1250225397; font-weight:600;  }
	.2852455533{ font-size: 35px; color: #80ceff; font-family:1250225397; font-weight:600;  }
	.2773066744{ font-size: 34px; color: #fce8b8; font-family:3448399207; font-weight:600;  }
	.1983279013{ font-size: 48px; color: #70584a; font-family:3448399207;  }
	.3133077588{ font-size: 44px; color: #70584a; font-family:1250225397; font-weight:700;  }
	.476258993{ font-size: 42px; color: #70584a; font-family:1250225397; }
	.349972590{ font-size: 30px; color: #78624a; font-family:1250225397; font-weight:600;  }
	.2212953428{ font-size: 30px; color: #FFFFFF;font-family:3448399207; }
	.3309993642{ font-size: 34px; color: #FFFFFF;font-family:3448399207; }
	.3897935017{ font-size: 30px; color:#FFF527;font-family:3448399207; }
	.1370102707{font-size: 36px; color:#379b3b; font-family:1250225397; font-weight:600; }
	.4082442381{font-size: 44px; color:#6b6b6b; font-family:3448399207;font-weight:600; }
	.2992659532{font-size: 44px; color:#52784a; font-family:3448399207;font-weight:600; }
	.1988132847{font-size: 44px; color:#4a5778; font-family:3448399207;font-weight:600; }
	.3406526789{font-size: 44px; color:#644a78; font-family:3448399207;font-weight:600; }
	.2343467686{font-size: 44px; color:#78624a; font-family:3448399207;font-weight:600; }
	.3365988401{font-size: 44px; color:#784a4a; font-family:3448399207;font-weight:600; }
	.899816536{font-size: 44px; color:#555555; font-family:3448399207;font-weight:600; }
	.2879054713{font-size: 44px; color:#416549; font-family:3448399207;font-weight:600; }
	.476646489{font-size: 44px; color:#46516d; font-family:3448399207;font-weight:600; }
	.3728724635{font-size: 44px; color:#773f76; font-family:3448399207;font-weight:600; }
	.2700776097{font-size: 44px; color:#b27f11; font-family:3448399207;font-weight:600; }
	.3254672828{font-size: 44px; color:#763636; font-family:3448399207;font-weight:600; }
	.2680331799{ font-size: 40px; color:#78624a; font-family:1250225397; font-weight:600; }
	.1276987957{ font-size: 40px; color:#78624a; font-family:1250225397; font-weight:600; }
	.3533372221{ font-size: 38px; color:#b27411; font-family:1250225397; font-weight:600; }
	.2859460019{ font-size: 32px; color:#198e92; font-family:1250225397; font-weight:600; }
	.3590894700{color: #64615d;font-size: 42px; font-family:1250225397; font-weight:600;}
	.3980531333{color: #fff;font-size: 54px; font-family:1250225397; font-weight:600;}
	.2519524076{color: #fff;font-size: 62px; font-family:1250225397; font-weight:600;}
	.3289029601{color: #78624a;font-size: 40px; font-family:1250225397;}
	.1622389633{color: #705a42;font-size: 40px; font-family:1250225397;}
	.2460833844{ font-size: 36px; color: #725432; font-family:1250225397; }
	.2298924266{color: #78624a;font-size: 40px;font-family:1250225397;font-weight:600;}
	.596094025{color: #78624a;font-size: 40px;font-family:1250225397;font-weight:bold;}
	.2577202698{color: #858484;font-size: 40px;font-family:1250225397;}
	.3100000329{color: #fffae8;font-size: 32px;font-family:3448399207;}
	
	.1913721540{ font-size: 38px; color: #fff5ed; font-family:1250225397; font-weight:700; text-shadow: 0px 1px 1px #000000} 
	.4129366814{ font-size: 34px; color: #ffe8db; font-family:3448399207; text-shadow:0px 1px 3px #000000 }
	.1330734894{ font-size: 38px; color: #593434; font-family:1250225397; font-weight:600;  }
	.444331298{ font-size: 32px; color: #725432; font-family:3448399207; text-stroke: 2px #e1bf76; }
	.3599109491{ font-size: 38px; color: #345359; font-family:1250225397; font-weight:600;  }
	.554990346{ font-size: 38px; color: #ffeab8; font-family:1250225397; font-weight:700; text-shadow: 0px 1px 3px #000000}
	.2634663264{ font-size: 38px; color: #ffe38c; font-family:3448399207;}
	.4113394870{ font-size: 38px; color: #ffe38c; font-family:3448399207;text-gradient: linear-gradient(0deg, #fef7e0 0%, #ffffff 100%);text-stroke:4px #917035;}
	.745102761{ font-size: 50px; color: #fffcdc; font-family:3448399207; font-weight:600;  text-gradient: linear-gradient(0deg,#fff8df 20%, #ffd58b 80% );text-shadow:2px 2px 2px #fc3b00}
	.1728979232{ font-size: 36px; color: #a64141; font-family:1250225397; font-weight:600;  }
	.55322058{ font-size: 32px; color: #a64141; font-family:1250225397; font-weight:600;  }
	.741638935{ font-size: 42px; color: #ffe9c3; font-family:3448399207; text-shadow: 0px 2px 3px #000000; text-gradient: linear-gradient(21deg, #ffd798, #ffe3b6 68%)}
	.2224903838{ font-size: 40px; color: #ffe9c3; font-family:3448399207; text-shadow: 0px 2px 5px rgba(0,0,0,0.315); text-gradient: linear-gradient(69deg, #ffd798, #ffe3b6 68%)}
	.4245993559{ font-size: 40px; color: #e9e4ae; font-family:3448399207; text-stroke: 4px #324250; }
	.106396305{ font-size: 40px; color: #d65446; font-family:1250225397; font-weight:600;  text-stroke: 4px #503004; text-shadow:2px 3px 2px #2d1e1e}
	.696747942{ font-size: 40px; color: #ffdd3e; font-family:1250225397; font-weight:600;  text-stroke: 4px #503004; text-shadow: 2px 3px 2px #2d2a1e}
	.238430353{ font-size: 40px; color: #46b6d6; font-family:1250225397; font-weight:600;  text-stroke: 4px #503004; text-shadow: 2px 3px 2px #2d1e1e }
	.3113240876{ font-size: 40px; color: #4f85f1; font-family:1250225397; font-weight:600;  text-stroke: 4px #503004; text-shadow: 2px 3px 2px #1e222d}
	.85322078{ font-size: 40px; color: #fefff4; font-family:1250225397; font-weight:600;  text-stroke: 4px #393d33; text-shadow:2px 3px 2px #1e282d }
	.3870890473{ font-size: 40px; color: #eed67d; font-family:1250225397; font-weight:600;  text-stroke: 4px #393d33; text-shadow: 2px 3px 2px #2d1e1e}
	.2887578273{ font-size: 54px; font-family:3448399207; color: #e6dcb3; text-stroke: 4px #544b43; }
	.3166786334{ font-size: 48px; font-family:3448399207; color: #e6dcb3; text-stroke: 2px #544b43; }
	.1762502372{ font-size: 42px; font-family:3448399207; color: #ffeab8; text-shadow:0px 1px 0.5px #000000; }
	.1511930306{ font-size: 64px; color: #e6dcb3; font-family:3448399207; text-gradient: linear-gradient(0deg, #e4eac5, #e6dcb3); text-shadow: 0px 2px 3px rgba(57,77,112,1); text-stroke: 4px #544b43; } 
	.90095291{ font-size: 49px;color: #fffbdd; font-family:1250225397; font-weight:600; text-gradient: linear-gradient(0deg, #977b39, #796346) }
	.2576307937{ font-size: 40px;color: #fffbdd; font-family:1250225397; font-weight:600; text-gradient: linear-gradient(0deg, #977b39, #796346) }
	.4084139072{ font-size: 32px; font-family:1250225397; font-weight:600;  color: #ffffff; text-stroke: 2px #434343; }
	.1325072324{ font-size: 32px; font-family:1250225397; font-weight:600;  color: #ffffff; text-stroke: 2px #434343; } 
	.4248557454{ font-size: 28px; font-family:1250225397; font-weight:600;  color: #ffffff; text-stroke: 3px #434343; } 
	.4139886167{ font-size: 34px; font-family:3448399207; color: #ffffff; text-gradient: linear-gradient(0deg, #fffae4, #ffcf71 70%);font-weight:600; } 
	.1931904439{ font-size: 36px; font-family:3448399207; color: #e9e4ae; text-stroke: 4px #324250; }
	.3791419272{ font-size: 38px; font-family:1250225397; font-weight:600;  color: #ffffff; text-stroke: 4px #693f25; }
	.3643725670{ font-size: 42px; font-family:1250225397; font-weight:600;  color: #ffeab8;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);}
	.3324345056{ font-size: 32px; font-family:1250225397; color: #ffeab8;}
	.4267427587{ font-size: 36px; font-family:1250225397; color: #ffeab8;}
	.604100463{ font-size: 36px; font-family:1250225397; font-weight:600;  color: #f9f5e2;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);}
	.370157493{ font-size: 32px; font-family:1250225397; font-weight:600;  color: #f9f5e2;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);}
	.4188146577{ font-size: 40px; font-family:1250225397; font-weight:600;  color: #ffeab8;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);}
	.3684182747{ font-size: 40px; font-family:1250225397; font-weight:600;  color: #7bde0a;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);}
	.2088555577{ font-size: 36px; font-family:1250225397; font-weight:600;  color: #7bde0a;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);}
	.3715616628{ font-size: 32px; font-family:1250225397; font-weight:600;  color: #7bde0a;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);}
	.2967837997{ font-size: 48px; font-family:1250225397; font-weight:600;  color: #4e3e29; }
	.1952092733{ font-size: 48px; font-family:1250225397; font-weight:600;  color: #8f0d0d; }
	.1721492339{ font-size: 50px; font-family:3448399207; color: #fff5e0;text-stroke: 4px #b98320;text-gradient: linear-gradient(0deg, #fff7cf, #f3ba4c 70%);font-weight:600; }
	.2430264879{ font-size: 74px; font-family:3448399207; color: #ff8a33;text-gradient: linear-gradient(0deg, #fff7de 20%, #ffd385 70%);font-weight:600; }
	.825818457{ font-size: 38px; font-family:1250225397; font-weight:600;  color: #ffffff;text-stroke:3px #000000;font-weight: bold; }
	.1767169387{ font-size: 38px; font-family:1250225397; font-weight:600;  color: #3dcb62;text-stroke:4px #302315;font-weight: bold; }
	.495396696{ font-size: 38px; font-family:1250225397; font-weight:600;  color: #ff3833;text-stroke:4px #302315;font-weight: bold; }
	.3604513657{ font-size: 42px; font-family:3448399207; color: #ffe9c3;text-gradient: linear-gradient(0deg, #ffd798, #ffe3b6);text-shadow: 0px 2px 3px rgba(0,0,0,0.31); }
	.460558688{ font-size: 34px; font-family:3448399207; color: #ffeab8;text-shadow: 0px 1px 3px rgba(0,0,0,0.42); }
	.3089001179{ font-size: 34px; font-family:3448399207; color: #ffe3ae;text-shadow: 0px 1px 3px rgba(0,0,0,0.42);text-gradient: linear-gradient(0deg, #f3ba4c, #fff1ad); }
	.844885311{ font-size:80px; font-family:3448399207; text-gradient: linear-gradient(180deg, #ffb741 30%, #f7eec7 100%);text-shadow: 0px 0px 3px rgba(150,113,43,1); }
	.2301258633{ font-size: 40px; font-family:3448399207; color: #faf7e6;text-shadow: 0px 1px 0px rgba(0,0,0,0.3);text-gradient: linear-gradient(0deg, #ffecaa 30%, #fbf5dc); }
	.2278019420{font-size: 42px;font-family:3448399207; color: #fff;}
	.647777538{font-size: 40px;font-family:3448399207; color: #fff;font-weight:600;text-shadow: 0px 2px 0px rgba(124,73,12,0.5);text-stroke:2px #ad6b00;}
	.1503558068{font-size: 52px;font-family:3448399207; color: #fff;font-weight:600;text-shadow: 2px 2px 6px rgba(124,73,12,0.5);text-stroke:2px #9e6d4b;}
	.3815431417{font-size: 40px;font-family:3448399207; color: #d3c3c3;text-stroke:2px #846432;}
	.2875465346{font-size: 40px;font-family:3448399207; color: #fff;text-stroke:2px #d98a80;}
	.3147855332{font-size: 34px;font-family:3448399207; color: #78624a;}
	
	.4053338583{ font-size: 32px; font-family:3448399207; color: #636363;text-gradient: linear-gradient(0deg, #636363 0%, #4d4d4d 100%); }
	.2586410293{ font-size: 42px; font-family:3448399207; color: #636363;text-gradient: linear-gradient(0deg, #636363 0%, #4d4d4d 100%); }
	.841513070{ font-size: 32px; font-family:3448399207; color: #fff; }
	.2639343304{ font-size: 34px; font-family:3448399207; color: #f2dbbe;font-weight: bold; }
	.2170858613{ font-size: 36px; font-family:3448399207; color: #ffeab8;text-shadow: 0px 1px 3px rgba(0,0,0,0.42); }
	.1676913241{font-size: 44px;font-family:3448399207;font-weight: bold;color: #904933;text-stroke: 2px #EDD7B1;}
	.3527658515{ font-size: 34px; font-family:1250225397; color: #ffeab8;text-shadow: 0px 1px 3px rgba(0,0,0,0.42); }
	.1833968667{font-size: 40px;font-family:1250225397; font-weight:600; color: #624D21;}
	.1424895339{ font-size: 42px;font-family:1250225397; font-weight: bold;color: #E1CA86;line-height: 32px;}
	.2475219418{font-size: 32px;font-family:3448399207; font-weight: 400;color: #FCE8B8;line-height: 26px;}
	.2969645185{font-size: 38px;font-family:1250225397; font-weight:600; color: #9BBCE9;line-height: 37px;}
	.936650864{font-size: 38px;font-family:1250225397; font-weight:600; color: #9BBCE9;line-height: 24px;}
	.1467777676{font-size: 32px;font-family:1250225397; font-weight:600; color: #70584A;line-height: 45px;}
	.3959403723{font-size: 44px;font-family:1250225397; font-weight:600; color: #78624A;}
	.1687202364{font-size: 42px;font-family:1250225397; font-weight:600; color: #ae7b31;}
	.813876435{font-size: 40px;font-family:3448399207;color: #cb7a39;}
	.4050791991{font-size: 38px;font-family:1250225397; font-weight:600; color: #9f6b20;}
	.1449907806{font-size: 38px;font-family:1250225397; font-weight:600; color: #51965a;}
	.1882382193{font-size: 36px;font-family:1250225397; font-weight:600; color: #78624a;}
	.235110106{font-size: 35px;font-family:1250225397; color: #fff;}
	.581998256{font-size: 35px;font-family:1250225397; color: #979797;}
	.863186819{font-size: 35px;font-family:1250225397; color: #3dcb62;}
	.3140573034{font-size: 32px;font-family:1250225397; color: #3dcb62;}
	.2785659468{font-size: 32px;font-family:1250225397; color: #979797;}
	.2884514916{font-size: 38px;font-family:1250225397; font-weight:600; color: #a64141;}
	.2046557296{font-size: 38px;font-family:1250225397; font-weight:600; color: #c18400;}
	.793151210{font-size: 34px;font-family:1250225397; font-weight:600; color: #a64141;}
	.2707239860{font-size: 34px;font-family:1250225397; font-weight:600; color: #34a5a9;}
	.1161544268{font-size: 36px;font-family:1250225397; font-weight:600; color: #676e48;}
	.3128293642{font-size: 40px;font-family:1250225397; color: #78624A;}
	.1053381418{font-size: 32px;font-family:1250225397; color: #a64141;}
	
	.1174309917{font-size: 32px;font-family:1250225397; font-weight:600; color: #78624a;}
	.2085625629{font-size: 36px;font-family:1250225397; font-weight:600; color:#78624a;}
	.862443921{font-size: 36px;font-family:1250225397; font-weight:600; color:#a64141;}
	.3706741148{font-size: 34px;font-family:1250225397; font-weight:600; color: #51965a;}
	.3311193928{font-size: 34px;font-family:1250225397; font-weight:600; color: #965151;}
	.1995194715{font-size: 28px;font-family:1250225397; font-weight:600; color: #78624a;}
	.2349042690{font-size: 36px;font-family:1250225397; font-weight:600; color: #78624A;}
	.3804836836{font-size: 34px;font-family:1250225397; font-weight:600; color: #78624A;}
	 
	.4032608042{font-family:1250225397;color: #70584a;font-size: 40px;}
	.2910190760{font-family:1250225397;color: #70584a;font-size: 36px;}
	.4130960565{font-family:1250225397;color: #fff;font-size: 32px;}
	.1821520343{font-family:1250225397;color: #7b654c;font-size: 32px;}
	.1334386461{font-family:1250225397;color: #ede3cb;font-size: 32px;}
	.621065609{font-family:1250225397;color: #907256;font-size: 32px;}
	.1197872586{font-family:1250225397;color: #4f824d;font-size: 32px;}
	.111631319{font-family:1250225397;color: #000000;font-size: 32px;}
	.3313925913{ font-family:1250225397; font-weight:600; color: #78624a; font-size: 30px; }
	.4234082919{ font-family:1250225397; font-weight:600; color: #78624a; font-size: 38px; }
	.2170846785{ font-family:1250225397; font-weight:600; color: #78624a; font-size: 42px; }
	.3934272346{ font-family:1250225397; font-weight:600; color: #51965a; font-size: 32px; }
	.4123791961{ font-family:1250225397; font-weight:600; color: #51965a; font-size: 42px; }
	.3824071033{ font-family:1250225397; font-weight:600; color: #a64141; font-size: 30px; }
	.2595748576{ font-family:1250225397; font-weight:600; color: #a64141; font-size: 38px; }
	.7788338{ font-family:1250225397; font-weight:600; color: #a64141; font-size: 42px; }
	.585878048{ font-family:1250225397; font-weight:600; color: #fcfee7; font-size: 34px; }
	.1156861198{font-family:1250225397; color: #4f824d; font-size: 46px;}
	.572079029{font-family:1250225397; color: #78624a; font-size: 46px;}
	.2560663831{font-family:1250225397;color: #6d5053;font-size: 52px;}
	.3678177023{font-family:1250225397;color: #a16137;font-size: 40px; font-weight:600;}
	.1774245970{font-family:1250225397;color: #e5d9b5;font-size: 30px;}
	.2548931677{font-family:1250225397; color: #cb8d0b; font-size: 40px; font-weight:600;}
	.2043636526{font-family:1250225397; color: #9d9d9d; font-size: 42px; font-weight:600;}
	.3497532862{font-family:1250225397; color: #fff; font-size: 42px; font-weight:600;}
	.863752440{font-family:3448399207; color: #fff; font-size: 40px;}
	.2870264752{font-family:1250225397; color: #fff; font-size: 40px;}
	.2682619150{font-family:1250225397; color: #fff; font-size: 46px; font-weight:600;}
	.3669851547{font-family:1250225397; color: #fff; font-size: 38px; font-weight:600;}
	.1306970686{font-family:1250225397; color: #fff; font-size: 36px; font-weight:600;}
	.1580259250{font-family:1250225397; color: #6bd7ff; font-size: 38px; font-weight:600;}
	.3186636951{font-family:1250225397; color: #43c448; font-size: 36px;}
	.1919593708{font-family:1250225397; color: #fff; font-size: 36px;}
	.869278993{font-family:1250225397; color: #b6b6b6; font-size: 36px;}
	.1030297331{text-stroke: 2px #4a4846;font-family:1250225397;color: #fbf6c2;font-size: 36px;}
	.2814836523{color: #70584a;font-size: 32px;font-family:1250225397; font-weight:600;}
	.3721544129{color: #4a8057;font-size: 32px;font-family:1250225397; font-weight:600;}
	.2925605207{color: #884233;font-size: 36px;font-family:1250225397; font-weight:600;}
	.172199041{color: #43c448;font-size: 36px;font-family:1250225397; font-weight:600;}
	.1189620391{color: #b6b6b6;font-size: 36px;font-family:1250225397; font-weight:600;}
	.3147700573{color: #fffefa;font-size: 30px;font-family:1250225397;font-weight: bold;}
	.4005740966{color: #43c448;font-size: 36px;font-family:1250225397;}
	.2796901092{color: #fff;font-size: 36px;font-family:1250225397;}
	.3666319209{color: #3dcb62;font-size: 36px;font-family:1250225397;}
	.3020900383{color: #85715a;font-size: 42px;font-family:1250225397; font-weight:600;}
	.2584818299{color: #884233;font-size: 35px;font-family:1250225397; font-weight:600;}
	.2706931877{color: #70584a;font-size: 40px;font-family:1250225397; font-weight:600;}
	.1172224054{color: #4f824d;font-size: 38px;font-family:1250225397; font-weight:600;}
	.1087035673{color: #70584a;font-size: 40px;font-family:1250225397;}
	.2271348965{color: #4a8057;font-size: 38px;font-family:1250225397;}
	.2002805644{color: #fffefa;font-size: 30px;font-family:1250225397;font-weight: bold;}
	.2191520111{color: #fffefa;font-size: 60px;font-family:1250225397;}
	.3721481936{color: #fffefa;font-size: 30px;font-family:1250225397;font-weight:bold;}
	.1703942789{color: #fffefa;font-size: 36px;font-family:1250225397;}
	.1761275428{color: #c7e7f1;font-size: 46px;font-family:3448399207; }
	.243200250{color: #fbe1da;font-size: 46px;font-family:3448399207; }
	.2014021181{color: #626c62;font-size: 34px;font-family:1250225397;}
	.3928134456{color: #fffefa;font-size: 60px;font-family:1250225397;}
	.1111750611{color: #fffefa;font-size: 52px;font-family:1250225397;}
	.2291773356{color: #faf4dd;font-size: 36px;font-family:1250225397;text-gradient: linear-gradient(270deg, #ffcf71, #fffae4 20%);}
	.3198487965{color: #78624a;font-size: 38px;font-family:1250225397;}
	.3311801025{color: #6b4e2e;font-size: 30px;font-family:1250225397;}
	.3331103554{color: #676e48;font-size: 32px;font-family:3448399207;}
	.1436886837{color: #78624a;font-size: 34px;font-family:1250225397;}
	.47193574{color: #78624a;font-size: 40px;font-family:1250225397;}
	.2396647126{color: #964dce;font-size: 36px;font-family:1250225397;}
	.1785331823{color: #b36d22;font-size: 34px;font-family:1250225397;}
	.2776692955{color: #fff;font-size: 34px;font-family:1250225397;}
	.3206965079{color: #707070;font-size: 34px;font-family:1250225397;}
	.1522147046{color: #78624a;font-size: 38px;font-family:1250225397;}
	.3278350389{color: #fcfee7;font-size: 38px;font-family:1250225397;}
	.4115365235{font-size: 36px;color: #fff;font-family:1250225397;}
	.1807911151{color: #ffffff;font-size: 26px;font-family:3448399207;}
	
	.4188290160{font-size: 38px;font-family:3448399207; color: #e0f4ff;}
	.3087407632{color: #78624a;font-size: 50px;font-family:3448399207;}
	.4101876198{color: #78624a;font-size: 44px;font-family:3448399207;}
	.2656656598{color: #70584a;font-size: 32px;font-family:3448399207;}
	.2352383738{font-size: 46px;color: #fff;font-family:3448399207;}
	.425974430{color: #9f6b20;font-size: 46px;font-family:3448399207;}
	.3251280803{color: #9f6b20;font-size: 36px;font-family:3448399207;}
	.2196321657{color: #78624a;font-size: 36px;font-family:3448399207;}
	.367376553{color: #78624a;font-size: 32px;font-family:3448399207;}
	.442751157{color: #d19a53;font-size: 60px;font-family:3448399207;}
	.1455775920{color: #efecd8;font-size: 42px;font-weight: 600;font-family:3448399207;text-stroke:4px #a8543b;}
	.3120979912{font-family:3448399207;color: #78624a;font-size: 56px;}
	.2644223161{ font-family:3448399207; color: #78624a; font-size: 38px; }
	.2245296188{ font-family:3448399207; color: #78624a; font-size: 32px; }
	.644561781{ font-family:3448399207; color: #c4e3f6; font-size: 38px; }
	.194094537{ font-family:3448399207; color: #ffffff; font-size: 38px; }
	.1543865867{ font-family:3448399207; color: #c4e3f6; font-size: 48px; }
	.3419927533{font-size: 40px;font-family:3448399207;color: #e0f4ff;}
	.3716474888{ font-family:3448399207; color: #78624a; font-size: 46px; }
	.1273547520{ font-family:3448399207; color: #ffffff; font-size: 36px; }
	.3620994925{font-family:3448399207; font-size: 50px; color: #7d766e;}
	.1851337537{ font-family:3448399207; color: #bd6363; font-size: 44px; font-weight:600;}
	.3541734412{font-family:3448399207; color: #c28579; font-size: 52px; font-weight:600;}
	.3427541650{font-family:3448399207; color: #ba796d; font-size: 50px; font-weight:600;}
	.2697173593{font-family:3448399207; color: #76ae67; font-size: 40px; font-weight:600;}
	.3744278530{font-family:3448399207; color: #fff; font-size: 48px; font-weight:600;}
	.1412255698{font-family:1250225397; color: #fff; font-size: 34px;}
	.1498118557{font-family:3448399207; color: #fff2b7; font-size: 56px; font-weight:600;text-gradient: linear-gradient(0deg, #fff9de, #fff2b7);text-stroke:3px #000000;}
	.3587362927{font-family:3448399207; color: #96d4ff; font-size: 56px; font-weight:600;text-gradient: linear-gradient(0deg, #ddf1ff, #96d4ff);text-stroke:3px #000000;}
	.3569307409{font-family:1250225397; color: #fffffe; font-size: 68px; font-weight:600;text-gradient: linear-gradient(0deg, #eff5ff, #c9deff);text-stroke:9px #93bcff80;}
	.1260184701{color: #78624a;font-size: 46px;font-family:3448399207; font-weight:600;}
	.608311264{color: #ffa92d;font-size: 80px;font-family:3448399207; font-weight:600;text-gradient: linear-gradient(0deg, #ffa92d, #f6eecb);text-stroke:2px #a57f3c;}
	.2354885116{color: #fffcf6;font-size: 42px;font-family:3448399207;font-weight:600;text-gradient: linear-gradient(0deg, #fffcf6, #ffd1ab);}
	.2037216393{color: #997c5d;font-size: 46px;font-family:3448399207;}
	.162917013{color: #78624a;font-size: 36px;font-family:3448399207;}
	.4264648059{color: #856947;font-size: 44px;font-family:3448399207;}
	.3102975734{color: #798ec2;font-size: 52px;font-family:3448399207;}
	.1193056859{color: #fff;font-size: 42px;font-family:3448399207;}
	.3403791291{color: #78624a; font-size: 46px;font-family:3448399207;}
	.4177434646{color: #e6dcb3; font-size: 46px; font-family:3448399207; text-stroke: 2px #544b43;}
	.1559442519{color: #fff;font-size: 46px;font-family:3448399207;}
	.4022937783{color: #e6dcb3;font-size: 50px;font-family:3448399207;text-stroke:2px #544b43;}
	
	.203577634{font-family:3448399207;color: #64615D;font-size: 36px;line-height: 40px;}
	
	.932855400{font-family:1250225397;;font-size: 26px;font-weight:600}
	
	.2864974803{color: #fff8ee;font-size: 24px;font-family:1250225397;}
	.3418744363{ font-family:1250225397; color: #70584a; font-size: 38px; }
	.3401507410{ font-family:1250225397; color: #70584a; font-size: 32px; }
	.2617554876{ font-family:1250225397; color: #70584a; font-size: 30px; }
	.1156888956{ font-family:1250225397; color: #b26e0f; font-size: 32px; }
	.2380308903{ font-family:3448399207; color: #d19a53; font-size: 58px; }
	.2432432395{ font-family:3448399207; color: #faf4dd; font-size: 36px;text-gradient: linear-gradient(0deg, #fffae4 10%, #ffcf71); }
	.923533112{ font-family:3448399207; color: #ffffff; font-size: 42px;text-gradient: linear-gradient(0deg, #ffffff 10%, #ffcf71); }
	.2790145835{ font-family:3448399207; color: #ffffff; font-size: 59px;text-stroke: 8px #262e2c; }
	.480839072{ font-family:3448399207; color: #faf4dd; font-size: 34px;text-stroke: 4px #312e2e; }
	.3356111948{ font-family:3448399207; color: #fefff4; font-size: 41px;text-stroke: 4px #1a191f; }
	.494428540{ font-size:40px; color:#fcfee7; line-height:70px; font-family:3448399207;}
	.3512334352{ font-size:40px; color:#fcfee7; line-height:60px; font-family:1250225397;text-shadow: 0px 1px 2px #000000;}
	.376438450{ font-family:3448399207; color: #ffd593; font-size: 42px;font-weight:600;text-gradient: linear-gradient(0deg, #ffffff 10%, #ffd593); }
	
	.1338092469{ font-family:3448399207; color: #78624a; font-size: 40px;line-height: 40px; }
	.2142945707{ font-family:3448399207; color: #ffffff; font-size: 62px;}
	.2480858216{ font-family:3448399207; color: #fdedca; font-size: 32px;}
	.4013059660{ font-family:1250225397; color: #379b3b; font-size: 32px; font-weight:700;}
	.1099887164{ font-family:3448399207; color: #fef8e1; font-size: 48px;text-stroke: 4px #5d5067;}
	.2210900409{ font-family:1250225397; color: #7bde0a; font-size: 28px;font-weight:700;text-stroke: 2px #60472b;}
	.3110818348{ font-family:1250225397; color: #ffeab8; font-size: 28px;font-weight:700;text-stroke: 2px #60472b;}
	.1613740954{ font-family:1250225397; color: #ff6d50; font-size: 28px;font-weight:700;text-stroke: 2px #60472b;}
	.1327212878{ font-family:1250225397; color: #fbf6c2; font-size: 34px;text-stroke: 2px #4a4846;}
	.97526858{ font-family:3448399207; color: #78624a; font-size: 34px;}
	.2334450849{ font-family:3448399207; color: #c4e3f6; font-size: 36px;}
	.2238930753{ font-family:3448399207; color: #78624a; font-size: 41px;}
	.2156194399{font-family:1250225397; color: #78624a; font-size: 38px; font-weight:600;}
	.1278213419{font-family:3448399207; color: #78624a; font-size: 62px;}
	.4140816380{font-family:3448399207; color: #965151; font-size: 44px;}
	.907634011{font-family:1250225397; color: #78624a; font-size: 34px;}
	
	.952085672{font-family:3448399207; color: #e6dcb3; font-size: 56px;text-gradient: linear-gradient(0deg, #ffe6ba, #fff8df);text-stroke: 2px #362b2b;}
	.438906455{font-family:3448399207; color: #e6dcb3; font-size: 56px;text-gradient: linear-gradient(0deg, #9abff7, #f0f6ff);text-stroke: 2px #362b2b;}
	.2584933839{font-family:3448399207; color: #e6dcb3; font-size: 56px;text-gradient: linear-gradient(0deg, #8c5cb6, #e0c4ff);text-stroke: 2px #2f2b36;}
	.1491155848{font-family:3448399207; color: #e6dcb3; font-size: 56px;text-gradient: linear-gradient(0deg, #a64040, #ffc4c4);text-stroke: 2px #2f2b36;}
	.2768952471{font-family:3448399207; color: #ffffff; font-size: 90px;text-gradient: linear-gradient(0deg, #ffffff, #d4cdb4);text-stroke: 6px #60472b;}
	.1379852092{font-family:3448399207; color: #5b698a; font-size: 48px;text-gradient: linear-gradient(0deg, #f6e9b0 0%, #e3b765 100%);}
	.1267087065{font-family:1250225397; color: #5b698a; font-size: 32px; line-height: 50px; text-gradient: linear-gradient(0deg, #f6e9b0 10%, #efcb88 57%, #d7b474 90%);}
	.2670505297{font-family:3448399207; color: #ffffff; font-size: 22px;text-stroke: 5px #472d11;}
	.3386937405{font-family:1250225397; color: #ffc776; font-size: 26px;}
	.662661778{font-family:3448399207; color: #ffc776; font-size: 32px;text-gradient: linear-gradient(0deg, #f6e9b0 10%, #efcb88 57%, #d7b474 90%);}
	.2217952892{font-family:1250225397; color: #78624a; font-size: 36px;font-weight: 700;}
	.821647459{font-family:3448399207; color: #78624a; font-size: 36px;}
	.1753738604{font-family:1250225397; color: #78624a; font-size: 34px;font-weight: 700;}
	.3961890528{font-family:1250225397; color: #78624a; font-size: 34px;}
	.393242666{font-family:1250225397; color: #78624a; font-size: 32px;}
	.273250713{font-family:1250225397; color: #af6701; font-size: 32px;}
	.3450006657{font-family:1250225397; color: #78624a; font-size: 36px;}
	.527152568{font-family:1250225397; color: #9b4e47; font-size: 32px;}
	.2402768126{font-family:1250225397; color: #b36d22; font-size: 32px;}
	.2572798214{font-family:1250225397; color: #f0dbab; font-size: 36px;font-weight: 700;}
	.1149509805{font-family:3448399207; color: #373638; font-size: 44px;font-weight: 700;}
	.2158545181{font-family:1250225397; color: #8b6b50; font-size: 40px;font-weight: 600;}
	.1269670378{font-family:1250225397; color: #d6d2c1; font-size: 38px;}
	.1092955744{font-family:3448399207; color: #494c49; font-size: 124px;}
	.1526994590{font-family:3448399207; color: #78624a; font-size: 40px;}
	.1096091694{font-family:3448399207; color: #ffffff; font-size: 40px;}
	.4183665657{font-family:3448399207; color: #70584a; font-size: 38px;}
	.339469717{font-size: 32px;color: #707070;font-family:1250225397;}
	.3694508548{font-size: 50px;color: #78624a;font-family:3448399207;}
	.3917916598{font-size: 34px;color: #884233;font-family:1250225397;}
	.2517804137{font-size: 46px;color: #ffffff;font-family:3448399207;}
	.2379100572{font-size: 42px;color: #78624a;font-family:3448399207;}
	.3289029601{font-size: 40px;color: #70584a;font-family:1250225397;}
	.1387315204{font-size: 30px;color: #ffffff;font-family:1250225397;}
	.850776391{font-size: 38px;color: #707070;font-family:1250225397;}
	.2603130547{font-size: 40px;color: #78624a;font-family:1250225397;}
	.3450006657{font-size: 36px;color: #70584a;font-family:1250225397;}
	.2342827322{font-size: 34px;color: #8cf377;font-family:1250225397;}
	.307951734{font-size: 30px;color: #55d952;font-family:1250225397;}
	.1717954014{font-size: 38px;color: #e7e2ad;font-family:3448399207;}
	.3784015872{font-size: 38px;color: #b93832;font-family:1250225397;}
	.1159653508{font-size: 32px;color: #e9cb7a;font-family:1250225397;}
	.3041371089{font-size: 38px;color: #37e767;font-family:1250225397;}
	.2510390113{font-size: 32px;color: #ffffff;font-family:1250225397;text-stroke: 4px #4d3917;}
	.2171630660{font-size: 38px;color: #ffffff;font-family:1250225397;}
	.1589719357{font-size: 38px;color: #ff3939;font-family:1250225397;}
	.3870675860{font-size: 32px;color: #ff3434;font-family:1250225397;}
	.1053972737{font-size: 38px;color: #319748;font-family:1250225397;}
	.2019328660{font-size: 32px;color: #37e767;font-family:1250225397;}
	.527991243{font-size: 44px;color: #78624a;font-family:1250225397;}
	.1378689115{font-size: 44px;color: #78624a;font-family:3448399207;}
	.1035174800{font-size: 34px;color: #099513;font-family:1250225397;}
	.2922932164{font-size: 40px;color: #70584a;font-family:1250225397;}
	.2884796174{font-size: 38px;color: #70584a;font-family:3448399207;}
	.2897064416{color: #ffffff;font-size: 80px;font-family:3448399207;text-shadow: 2px 2px 4px rgba(181,61,35,0.5);}
	.2155027464{ font-family:3448399207;color: #e9e4ae;font-size: 46px;text-stroke: 3px #324250; }
	.1001149066{ font-family:3448399207;color: #e9e4ae;font-size: 36px;text-stroke: 2px #324250; }
	.666057233{font-family:1250225397;color: #fdf7e1;font-size: 40px;text-gradient: linear-gradient(0deg, #ffdc9c, #fff8df);text-stroke: 4px #7f4a2b;}
	.2130879757{font-family:3448399207;color: #fdf7e1; font-size: 46px; text-shadow: 0px 1px 2px rgba(127,74,43,0.5); text-gradient: linear-gradient(0deg, #ffdc9c, #fff8df);}
	.301907364{font-family:3448399207;color: #7f4a2b; font-size: 28px; }
	.3427333283{font-family:3448399207;color: #fdf7e1; font-size: 28px; }
	.3828885172{font-family:3448399207;color: #fdf7e1; font-size: 32px; text-stroke: 4px #7f4a2b;}
	.750264630{font-family:1250225397;color: #f5f9ff; font-size: 40px; text-stroke: 4px #325492;}
	.2306263477{font-family:3448399207;color: #fdf7e1; font-size: 32px; text-stroke: 4px #2b457f;}
	.478358451{font-family:3448399207;color: #ffebc6; font-size: 40px; text-shadow: 0px 0px 2px rgba(146,74,60,1); text-stroke: 4px #75572f;}
	.2968152677{font-family:3448399207;color: #ffebc6; font-size: 44px; text-shadow: 0px 0px 2px rgba(146,74,60,1); text-stroke: 4px #75572f;}
	.2719778711{font-family:3448399207;color: #ffebc6; font-size: 32px; text-shadow: 0px 0px 2px rgba(146,74,60,1); text-stroke: 4px #75572f;}
	.2164793845{font-family:3448399207;color: #ffebc6;font-size: 44px; text-gradient: linear-gradient(0deg, #ac681b, #845015);} 
	.38539485{font-family:3448399207;color: #e6dcb3;font-size: 44px; text-gradient: linear-gradient(0deg, #fff8df, #ffe6ba);} 
	.3564886306{font-family:3448399207;color: #ffebc6;font-size: 42px; text-gradient: linear-gradient(0deg, #ac681b, #845015);} 
	.952085672{font-family:3448399207;color: #e6dcb3;font-size: 56px; text-stroke: 2px #7b624c; text-gradient: linear-gradient(0deg, #ffe6ba, #fff8df);}
	.3931921676{font-family:3448399207;color: #78624a;font-size: 80px; text-gradient: linear-gradient(0deg, #f6eecb, #ffe98d, #ffa92d); text-shadow: 0px 0px 2px rgba(190,137,45,0.5);}
	.1133613501{ font-family:3448399207;color: #fffdd7; font-size: 36px; text-stroke: 4px #725d47;} 
	.164577048{ font-family:3448399207;color: #82e884; font-size: 36px; text-stroke: 4px #2c2c2c;} 
	.3117099399{ font-family:3448399207;color: #ffffff; font-size: 34px;}
	.702627336{ font-family:3448399207;color: #57ef62; font-size: 36px; text-stroke: 4px #2c5922; text-gradient: linear-gradient(0deg, #aaffa4, #ffffff)} 
	.3124852412{ font-family:3448399207;color: #ff6464; font-size: 36px; text-stroke: 4px #4e1d1d; text-gradient: linear-gradient(0deg, #ffaeae, #ffffff)} 
	.52865168{ font-family:3448399207;color: #78624a;font-size: 28px; }
	.2990035377{ font-family:3448399207;color: #c4e3f6;font-size: 40px;}
	.36287680{font-family:3448399207;color: #554225;font-size: 40px; text-gradient: linear-gradient(0deg, #916327, #574325);} 
	.3813489064{font-family:3448399207;color: #78624a;font-size: 56px;} 
	.959612168{font-family:3448399207;color: #fff;font-size: 36px;} 
	.3278350389{font-family:3448399207;color: #fff;font-size: 34px;} 
	.3633624115{font-family:3448399207;color: #856947;font-size: 42px;} 
	.3543320832{font-family:3448399207;color: #fff;font-size: 38px;text-gradient: linear-gradient(0deg, #ffd99f, #fff); text-stroke: 1px #ad5a3a;} 
	.750547963{font-family:3448399207;color: #665d4e;font-size: 34px;} 
	.1734121264{font-family:1250225397;color: #f9f5e2;font-size: 26px;} 
	
	.2660352803{ font-family:3448399207;font-size: 40px;color: #daf5f8; }
	.1527789966{ font-family:3448399207;font-size: 40px;color: #3a8130; }
	.2671606457{ font-family:3448399207;font-size: 50px;color: #3a8130; }
	
	.2989820574{color: #858484;font-size: 32px;font-family:1250225397;}
	.1530629069{color: #da0000;font-size: 32px;font-family:1250225397;}
	.3738520782{color: #78624a;font-size: 32px;font-family:1250225397;}
	.1732476222{color: #df4141;font-size: 32px;font-family:1250225397;}
	.620564471{font-size: 32px; color:#555555; font-family:1250225397; }
	.2268629695{font-size: 32px; color:#416549; font-family:1250225397; }
	.1374888517{font-size: 32px; color:#46516d; font-family:1250225397; }
	.2344036770{font-size: 32px; color:#773f76; font-family:1250225397; }
	.2737952717{font-size: 32px; color:#b27f11; font-family:1250225397; }
	.1758845384{font-size: 32px; color:#763636; font-family:3448399207; }
	.3635345670{color: #af6701;font-size: 32px;font-family:1250225397;}
	.1980232087{color: #fff;font-size: 36px;font-family:1250225397;line-height: 38px;}
	.1356403850{font-size: 32px;font-family:1250225397; color: #78624a;}
	.1969310424{font-size: 32px;font-family:1250225397; color:#78624a;}
	.1180292597{color: #78624a;font-size: 38px;font-family:1250225397;}
	.2601188000{ font-family:1250225397; color: #379b3b; font-size: 32px;}
	.4232573914{ font-family:1250225397; color: #70584a; font-size: 32px;}
	.591153975{ font-size: 34px; color: #70584a; font-family:1250225397;}
	.110727548{ font-family:1250225397; color: #b36d22; font-size: 34px;}
	.488391789{ font-family:1250225397; color: #8b745c; font-size: 34px;line-height: 40px; }
	.2602208882{color: #4f824d;font-size: 34px;font-family:1250225397;}
	.943530032{color: #70584a;font-size: 34px;font-family:1250225397;}
	.2722529405{color: #fffefa;font-size: 36px;font-family:1250225397;}
	.1097655213{color: #78624a;font-size: 32px;font-family:1250225397;}
	.596094025{color: #78624a;font-size: 32px;font-family:1250225397;}
	.1980484512{color: #70584a;font-size: 38px;font-family:1250225397;}
	.977667334{color: #64615d;font-size: 32px; line-height: 42px; font-family:1250225397;}
	.3423628757{ font-family:1250225397;font-size: 26px;color: #f9f5e2;}
	.1483001714{ font-family:1250225397;font-size: 26px;color: #e9cb7a;}
	.1892999855{ font-family:1250225397;font-size: 32px;color: #f9f5e2;}
	.3587016976{ font-family:1250225397;font-size: 32px;color: #f9f5e2;text-shadow: 1px 1px 0 rgba(0,0,0,0.42);}
	.1026242320{ font-family:1250225397;font-size: 40px;color: #957537;}
	.809876885{ font-family:1250225397;font-size: 32px;color: #78624a;}
	.902691900{ font-family:1250225397;font-size: 32px;color: #fffedc;text-stroke: 2px #735227;}
	.3469335986{ font-family:1250225397;font-size: 29px;color: #78624a;}
	.1679327608{ font-family:1250225397;font-size: 32px;color: #ffffff;}
	.308049096{ font-family:1250225397;font-size: 32px;color: #4f824d;}
	.2403114255{ font-family:1250225397;font-size: 34px;color: #78624a;}
	.1310650859{ font-family:1250225397;font-size: 36px;color: #78624a;}
	.564697228{ font-family:1250225397;font-size: 48px;color: #96502a;}
	.2505899937{ font-family:1250225397;font-size: 26px;color: #fffdf2;}
	.1083667533{ font-family:1250225397; font-size: 32px; color: #78624a; font-weight:600; }
	.1536789344{ font-family:1250225397; font-size: 26px; color: #78624a; font-weight:600; }
	.2774800991{ font-family:1250225397;font-size: 32px;color: #78624a;line-height: 36px;}
	.1893383455{ font-family:1250225397;font-size: 26px;color: #645a50;}
	.702229434{ font-family:1250225397;font-size: 26px;color: #645a50;line-height: 34px;}
	.4035104931{ font-family:1250225397;font-size: 26px;color: #645a50}
	.498819606{ font-family:1250225397;font-size: 28px;color: #645a50;}
	.1333510149{ font-family:1250225397;font-size: 32px;color: #645a50;}
	.2998308876{ font-family:1250225397;font-size: 32px;color: #21881c; }
	.3651752394{ font-family:1250225397;font-size: 26px;color: #21881c; }
	.2806511031{ font-family:1250225397;font-size: 32px;color: #884233; }
	.670497597{ font-family:1250225397;font-size: 32px;color: #fd3434; }
	.1348785758{color: #af6701;font-size: 32px;font-family:1250225397;}
	.3825178173{ font-family:1250225397;font-size: 26px;color: #ffffff;line-height: 56px;}
	.2606541004{ font-family:1250225397;font-size: 26px;color: #ffffff;}
	.1834554239{ font-family:1250225397;font-size: 26px;color: #78624a;}
	.2249156291{font-size: 28px;color: #dfedff;font-family:1250225397;}
	.2365370930{ font-family:1250225397;color: #ffffff; font-size: 30px;}
	.1250300428{ font-family:1250225397;font-size: 30px;color: #ffffff;}
	.3077221508{ font-family:1250225397;font-size: 32px;color: #ffffff;}
	.2331515628{ font-family:1250225397;font-size: 32px;color: #005ba1;}
	.3674154776{ font-family:1250225397;font-size: 34px;color: #423d36;}
	.2929115965{ font-family:1250225397;font-size: 32px;color: #ffffff;font-weight: bold;}
	.3604272345{ font-family:1250225397;font-size: 32px;color: #f9f5e2;}
	.2229807104{ font-family:1250225397;font-size: 32px;color: #ffffff;text-stroke: 2px #000000;}
	.3483992942{ font-family:1250225397;font-size: 32px;color: #f9f5e2;}
	.1700322289{ font-family:1250225397;font-size: 32px;color: #37e767;}
	.507938486{ font-family:1250225397;font-size: 32px;color: #37e767;font-weight: bold;}
	.3771021646{ font-family:1250225397;font-size: 32px;color: #6687eb;}
	.255616634{ font-family:1250225397;font-size: 32px;color: #c677fe;}
	.4220349504{ font-family:1250225397;font-size: 32px;color: #d88018;}
	.3055235704{ font-family:1250225397;font-size: 32px;color: #ff3434;}
	.1550088985{ font-family:1250225397;font-size: 32px;color: #b28ed4;}
	.217368718{ font-family:1250225397;font-size: 32px;color: #dcb86f;}
	.821402136{ font-family:1250225397;font-size: 32px;color: #df4141;}
	.4098827838{ font-family:1250225397;font-size: 32px;color: #a0a0a0;}
	.4144108277{ font-family:1250225397;font-size: 32px;color: #ff3434;}
	.2377696508{ font-family:1250225397;font-size: 32px;color: #4a8057;}
	.2167127308{ font-family:1250225397;font-size: 26px;color: #37e767;}
	.2360288892{ font-family:1250225397;font-size: 32px;color: #167abe;}
	.3744588002{ font-family:1250225397;font-size: 32px;color: #888684;}
	.3912981776{font-family:1250225397;font-size: 32px;color: #fffedc;text-stroke: 2px #735227;}
	.461353657{font-family:3448399207;font-size: 42px;color: #e9e4ae;text-stroke: 2px #314b77;}
	.3342255071{font-family:3448399207;font-size: 38px;color: #e9e4ae;text-stroke: 2px #314b77;}
	.3756198992{font-family:3448399207;font-size: 42px;color: #e9e4ae;text-stroke: 4px #314b77;}
	.2334829181{ font-family:1250225397;font-size: 32px;color: #e9cb7a;}
	.3397537091{ font-family:1250225397;font-size: 34px; color: #4e3e29; }
	.343555038{ font-family:1250225397;font-size: 34px; color: #345359; }
	.1160658002{ font-family:1250225397;font-size: 34px; color: #593434; }
	.3573661523{font-family:1250225397;font-size: 32px; color: #fff;text-stroke: 1px #ffeab8;}
	.2653296523{font-family:1250225397;font-size: 32px; color: #ffeab8;}
	.3527658515{font-family:1250225397;font-size: 32px; color: #ffeab8;text-shadow: 1px 0px 3px rgba(0,0,0,0.42);}
	.3212941838{font-family:1250225397;font-size: 32px; color: #a2ff6f;text-stroke: 1px #35b857;}
	.4012778871{font-family:1250225397;font-size: 32px; color: #9fdcff;text-stroke: 1px #40457e;}
	.1956672354{font-family:1250225397;font-size: 32px;color: #c677fe;text-stroke: 1px #6a11ce;}
	.972619295{font-family:1250225397;font-size: 32px; color: #ffd668;text-stroke: 1px #8b6c1a;}
	.3580761188{font-family:1250225397;font-size: 26px; color: #ffd668;text-stroke: 1px #8b6c1a;}
	.271480436{font-family:1250225397;font-size: 30px; color: #ffd668;text-stroke: 1px #8b6c1a;}
	.789676350{font-family:1250225397;font-size: 26px; color: #ffca9f;text-stroke: 1px #7e5540;}
	.493386735{font-family:1250225397;font-size: 26px; color: #9fdcff;text-stroke: 1px #40457e;}
	.3340658228{font-family:1250225397;font-size: 32px; color: #ff3434;text-stroke: 1px #771e1e;}
	.141415992{ font-family:1250225397;font-size: 34px; color: #ffffff; }
	.3888151126{ font-family:3448399207;font-size: 34px; color: #78624a; }
	.2544304793{ font-family:3448399207;font-size: 38px; color: #ffffff; }
	.2727819741{ font-family:3448399207;font-size: 44px; color: #665d4e; }
	.3273843644{ font-family:3448399207;font-size: 42px; color: #ffffff; }
	.2723142223{ font-family:3448399207;font-size: 42px; color: #c4e3f6; }
	.2140705270{ font-family:3448399207;font-size: 44px; color: #ffeab8;text-gradient: linear-gradient(0deg, #ffd58b 0%, #fff8df 100%);text-shadow: 0px 0px 3px rgba(0,0,0,0.42);text-stroke: 1px #70491a; }
	.979296169{ font-family:3448399207;font-size: 32px; color: #ffeab8;text-gradient: linear-gradient(0deg, #ffd58b 0%, #fff8df 100%);text-shadow: 0px 0px 3px rgba(0,0,0,0.42);text-stroke: 1px #70491a; }
	.1455775920{color: #efecd8;font-size: 42px;font-family:3448399207;text-stroke:4px #a8543b;}
	.172097077{font-family:4149012451; font-size: 130px; text-gradient: linear-gradient(180deg, #f2cf84 0%, #fffbd3 80%);text-shadow: 0px 0px 5px rgba(190,136,83,1); }
	.3356739349{font-family:4149012451; font-size: 110px; text-gradient: linear-gradient(180deg, #f2cf84 0%, #fffbd3 80%);text-shadow: 0px 0px 5px rgba(190,136,83,1); }
	.3698384477{font-family:4149012451; color: #ffffff; font-size: 42px;text-stroke: 2px #576267;}
	.1092760367{font-family:4149012451; color: #fffffe; font-size: 56px;font-weight:600;text-shadow:0px 0px 24px #ffd270;}
	.1351957833{font-family:4149012451; color: #ffffff; font-size: 83px; text-shadow: 0px 0px 2px rgba(109,146,202,1);}
	.4288309990{font-family:4149012451; color: #ffffff; font-size: 43px; text-shadow: 0px 0px 2px rgba(109,146,202,1);}
	.361379906{font-family:4149012451; color: #5b698a; font-size: 110px; text-gradient: linear-gradient(180deg, #adb5ff 0%, #f0f6ff 100%);text-shadow: 0px 0px 0px rgba(73,54,165,0.44);}
	.3928618244{font-family:4149012451; color: #5b698a; font-size: 110px; text-gradient: linear-gradient(180deg, #f2cf84 0%, #fffbd3 80%);text-shadow: 0px 0px 5px rgba(213,139,30,0.44);}
	.897571688{font-family:4149012451; color: #5b698a; font-size: 130px; text-gradient: linear-gradient(180deg, #f2cf84 0%, #fffbd3 64%);text-shadow: 0px 0px 16px rgba(190,136,83,1);}
	.3067790415{font-family:4149012451; color: #5b698a; font-size: 110px; text-gradient: linear-gradient(180deg, #f2cf84 0%, #fffbd3 80%);text-shadow: 0px 0px 5px rgba(213,139,30,0.44);}
	.2929261022{font-family:4149012451; color: #5b698a; font-size: 110px; text-gradient: linear-gradient(180deg, #f7e09f 0%, #fff5dd 100%);text-shadow: 0px 0px 5px rgba(213,139,30,0.44);}
	.2005382638{ font-size: 42px;color: #796346;font-family:4149012451;text-gradient: linear-gradient(0deg, #e3e0d6, #faf6de 2%, #faf6b1)}
	.3948324058{ font-size: 42px;color: #f4eeda;font-family:4149012451;text-gradient: linear-gradient(0deg, #977b39, #796346)}
	.2529275454{ font-size: 56px;color: #ffffff; font-family:3448399207; text-shadow: 0px 0px 2px rgba(147,188,255,0.8);}
	.3715533374{ font-size: 60px;color: #304766; font-family:3448399207; text-shadow: 0px 0px 7px rgba(202,103,24,1);text-gradient: linear-gradient(180deg, #f7e09f 0%, #fff5dd 100%);}
	.3304765149{ font-size: 60px;color: #b26d32; font-family:4149012451; text-shadow: 0px 0px 14px rgba(213,129,30,0.8);text-gradient: linear-gradient(180deg, #ffd96e, #fff5dd)}
	.3018010572{ font-size: 58px;color: #ffffff; font-family:3448399207; text-shadow: 0px 0px 18px rgba(68,68,68,1); }
	.4157480844{ font-size: 50px;color: #78624a; font-family:4149012451;}
	.1822350693{font-size: 48px; color: #e6dcb3; font-family:3448399207; text-gradient: linear-gradient(180deg, #ffe7ba 0%, #fff7de 100%);text-shadow: -1px -1px 5px rgba(255,255,255,0.28);text-stroke: 2px #7b624c;}
	.4202110399{font-size: 36px; color: #e6dcb3; font-family:3448399207; text-gradient: linear-gradient(180deg, #ffe7ba 0%, #fff7de 100%);text-shadow: -1px -1px 5px rgba(255,255,255,0.28);text-stroke: 2px #7b624c;}
	.987698338{font-size: 31px; color: #e6dcb3; font-family:3448399207; text-gradient: linear-gradient(180deg, #ffe7ba 0%, #fff7de 100%);text-shadow: -1px -1px 5px rgba(255,255,255,0.28);text-stroke: 2px #7b624c;}
	.3498943209{font-size: 48px; color: #e6dcb3; font-family:3448399207; text-gradient: linear-gradient(180deg, #ac681b 0%, #845015 100%);}
	.2927909057{font-size: 40px; color: #78624a; font-family:3448399207;}
	.4130960565{ font-size: 32px;color: #ffffff; font-family:1250225397;}
	.3084331176{font-family:4149012451; color: #425bd7; font-size: 64px;}
	.2486388054{font-family:4149012451; color: #008000; font-size: 64px;}
	.2405006977{font-family:4149012451; color: #da2626; font-size: 64px;}
	.3266624372{font-family:4149012451; color: #6e1c75; font-size: 64px;}
	.1480620723{font-family:4149012451; color: #724025; font-size: 64px;}
	.2978285354{font-family:4149012451; color: #146656; font-size: 64px;}
	.2666260360{font-family:4149012451; color: #b93fd7; font-size: 64px;}
	.1261434486{font-family:4149012451; color: #3a2843; font-size: 64px;}
	.111631319{font-family:1250225397; color: #000000; font-size: 32px;}
	.3144077675{font-family:1250225397; color: #ffffff; font-size: 60px;}
	.4113058757{ font-size: 46px;color: #ffffff; font-family:4149012451; }
	/* text-stroke 和 text-shadow 都可以实现字体加粗效果，其原理是颜色一致
	  https:
	*/
	
	
	
	
	
	.1250225397{ font-family:1250225397; font-weight:600;  } 
	
	.3448399207{ font-family:3448399207; }
	
	.4149012451{ font-family:4149012451; }
	
	
	.1355884763{ font-size:24px }
	.3444260523{ font-size:26px }
	.2929588175{ font-size:27.85px }
	.2643430591{ font-size:28px }
	.3798169775{ font-size:28.02px }
	.1472860779{ font-size:29.42px }
	.933756616{ font-size:30px }
	.2523023112{ font-size:30.18px }
	.2291754279{ font-size:30.33px }
	.1818270555{ font-size:30.61px }
	.351292394{ font-size:31.15px }
	.1895355457{ font-size:31.96px }
	.1447141507{ font-size:32px }
	.3586886673{ font-size:32.83px }
	.3512791527{ font-size:34px }
	.2211763961{ font-size:34.36px }
	.2395149787{ font-size:34.75px }
	.2758223301{ font-size:34.92px }
	.3669671433{ font-size:35px }
	.3436024508{ font-size:35.16px }
	.2527228330{ font-size:35.71px }
	.2550294270{ font-size:36px }
	.2225123230{ font-size:37.38px }
	.1092304116{ font-size:37.5px }
	.3840010901{ font-size:37.55px }
	.4005449474{ font-size:38px }
	.3784436350{ font-size:38.34px }
	.631264396{ font-size:38.47px }
	.2512135025{ font-size:39.21px }
	.3739134992{ font-size:39.61px }
	.2195538552{ font-size:40px }
	.2653521776{ font-size:40.83px }
	.865570928{ font-size:41.96px }
	.716649309{ font-size:42px }
	.2703693546{ font-size:43.3px }
	.1739480340{ font-size:46px }
	.4218345230{ font-size:48px }
	.3278008729{ font-size:52px }
	.3660952957{ font-size:56px }
	.2824853559{ font-size:59px }
	.3861399002{ font-size:77.1px }
	
	
	.2064322935{ color: #d5475a}
	.91296432{ color: #64615d}
	.2141331845{ color: #51965a}
	.2817152306{ color: #78624a}
	.3735530149{ color: #237139}
	.708470761{ color: #884233}
	.1227209348{ color: #4e3e29}
	.1477561563{ color: #a08350}
	.2136983568{ color: #3b2e20}
	.478752223{ color: #f9f4dd}
	.2715260154{ color: #c4e3f6}
	.1889728759{ color: #413e36}
	.2418802373{ color: #705a42}
	.2159711715{ color: #cfb46c}
	.178473412{ color: #4d7b9f}
	.3415123645{ color: #a64141}
	.1839605780{ color: #776048}
	.944092016{ color: #f6dac4}
	.3063410794{ color: #676e48}
	.3807537317{ color: #70584a}
	.1564646181{ color: #4d3e2d}
	.1743559685{ color: #379b3b}
	.3055857815{ color: #f2e3dc}
	.1584389455{ color: #f1eeda}
	.1899580996{ color: #d5e1ea}
	.3007279874{ color: #d5ead7}
	.216464088{ color: #e9cb7a}
	.3841131930{ color: #fffcf7}
	.1102570358{ color: #fcfee7}
	.804776018{ color: #c73a3a}
	.1013959792{ color: #49aa54}
	.630223480{ color: #e7dd9c}
	.1665347693{ color: #ffeab8}
	.1938316079{color: #478f36;}
	.231845572{color: #546a8b;}
	.2491577470{color: #8e66a4;}
	.2187940773{color: #c66151;}
	
	
	.2841610473{
	  position: absolute;
	  right: -6px;
	  top: -5px;
	  width: 100px;
	  height: 194px;
	  z-index: 2;
	}
	.45295019{
	  position:absolute;left:0;top:0;width:100%;height:100%;
	}
	.1622336806{
		position: absolute;
		width: 965px;
		height: 1426px;
		justify-content: center;
		align-items: center;
	}.4055802084{
		width: 100%;
		height: 100%;
		background-color: rgba(0,0,0,0.6);
	}
	
	.102134991{
		position: absolute;
		top: 50%;
		left: 50%;
		margin: -100px 0 0 -100px;
		width: 200px;
		height: 200px;
	}
	
	.1258688620{
		position: absolute;
		width: 30px;
		height: 30px;
		margin: -15px 0 0 -15px;
		border-radius: 15px;
	}
	@keyframes rotateAnima {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}";

	let r = match parse_class_map_from_string(r) {
        Ok(r) => match bincode::serialize(&r) {
            Ok(bin) => Result{err: None, bin: Some(bin)},
            Err(r) => Result{err: Some(r.to_string()), bin: None},
        },
        Err(r) => Result{err: Some(r), bin: None}
    };
	println!("===================={:?}", r);
	
}
