use crate::{Balance, Module, Trait};
use alloc::{string::String, vec::Vec};
use arrayvec::ArrayString;
use core::str;
use lite_json::json::JsonValue;
use vln_commons::{Asset, Collateral, PairPrice};

impl<T> Module<T>
where
    T: Trait,
{
    pub(crate) fn parse_btc_usd(json: &str) -> Option<PairPrice<Balance<T>>> {
        let has_correct_obj_name = |vec_chars: &Vec<char>, name: &str| {
            if vec_chars.iter().collect::<String>() == name {
                Some(())
            } else {
                None
            }
        };

        let [buy, sell] = match lite_json::parse_json(json).ok()? {
            JsonValue::Object(outer_obj) => {
                let outer_btc = outer_obj.get(0)?;
                has_correct_obj_name(&outer_btc.0, "BTC")?;
                let buy = match &outer_btc.1 {
                    JsonValue::Object(inner_obj) => {
                        let inner_usd = inner_obj.get(1)?;
                        has_correct_obj_name(&inner_usd.0, "USD")?;
                        match &inner_usd.1 {
                            JsonValue::Number(n) => Some(n.integer as u32),
                            _ => None,
                        }
                    }
                    _ => None,
                };

                let outer_usd = outer_obj.get(1)?;
                has_correct_obj_name(&outer_usd.0, "USD")?;
                let sell = match &outer_usd.1 {
                    JsonValue::Object(inner_obj) => {
                        let inner_btc = inner_obj.get(0)?;
                        has_correct_obj_name(&inner_btc.0, "BTC")?;
                        match &inner_btc.1 {
                            JsonValue::Number(n) => Some(n.integer as u32),
                            _ => None,
                        }
                    }
                    _ => None,
                };

                Some([buy?, sell?])
            }
            _ => None,
        }?;

        Some(PairPrice::new(
            [Asset::Btc, Asset::Collateral(Collateral::Usd)],
            buy.into(),
            sell.into(),
        ))
    }

    pub(crate) fn parse_usd_cop(html: &str) -> Option<PairPrice<Balance<T>>> {
        let banner_start = html.find("id=\"banner\"")?;
        let banner_start_str = html.get(banner_start..)?;
        let price = |search_for| {
            let search_for_start = banner_start_str.find(search_for)?;
            let search_for_str = banner_start_str.get(search_for_start..)?;
            let after_sign = search_for_str.split('$').nth(1)?;
            let price_str = after_sign.split("</h3>").next()?;
            Self::normalize_and_convert_number_str(price_str)
        };
        let buy = price("Te Compran")?;
        let sell = price("Te Venden")?;
        Some(PairPrice::new(
            [Asset::Collateral(Collateral::Usd), Asset::Cop],
            buy,
            sell,
        ))
    }

    fn normalize_and_convert_number_str(number_str: &str) -> Option<Balance<T>> {
        let mut buffer = ArrayString::<[u8; 128]>::new();
        let last_comma_idx_opt = number_str.rfind('.');
        if let Some(last_comma_idx) = last_comma_idx_opt {
            let (before, after) = number_str.split_at(last_comma_idx);
            for c in before.chars().filter(|c| c.is_numeric()) {
                buffer.push(c);
            }
            buffer.push('.');
            for c in after.chars().filter(|c| c.is_numeric()) {
                buffer.push(c);
            }
        } else {
            for c in number_str.chars().filter(|c| c.is_numeric()) {
                buffer.push(c);
            }
        }
        let n = buffer.parse::<u32>().ok()?;
        Some(n.into())
    }
}
