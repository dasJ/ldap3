use super::{ControlParser, MakeCritical, Oid};
use super::construct_control;

use bytes::BytesMut;

use asnom::common::TagClass;
use asnom::IResult;
use asnom::parse::{parse_uint, parse_tag};
use asnom::structure::StructureTag;
use asnom::structures::{ASNTag, Integer, OctetString, Sequence, Tag};
use asnom::universal::Types;
use asnom::write;

#[derive(Clone, Debug)]
pub struct PagedResults {
    pub size: i32,
    pub cookie: Vec<u8>,
}

pub const PAGED_RESULTS_OID: &'static str = "1.2.840.113556.1.4.319";

impl Oid for PagedResults {
    fn oid(&self) -> &'static str {
        PAGED_RESULTS_OID
    }
}

impl MakeCritical for PagedResults { }

impl From<PagedResults> for Option<Vec<u8>> {
    fn from(pr: PagedResults) -> Option<Vec<u8>> {
        let cookie_len = pr.cookie.len();
        let cval = Tag::Sequence(Sequence {
            inner: vec![
                Tag::Integer(Integer {
                    inner: pr.size as i64,
                    .. Default::default()
                }),
                Tag::OctetString(OctetString {
                    inner: pr.cookie,
                    .. Default::default()
                }),
            ],
            .. Default::default()
        }).into_structure();
        let mut buf = BytesMut::with_capacity(cookie_len + 16);
        write::encode_into(&mut buf, cval).expect("encoded");
        Some(Vec::from(&buf[..]))
    }
}

impl From<PagedResults> for StructureTag {
    fn from(pr: PagedResults) -> StructureTag {
        construct_control(PAGED_RESULTS_OID, false, pr.into())
    }
}

impl ControlParser for PagedResults {
    fn parse(val: &[u8]) -> PagedResults {
        let mut pr_comps = match parse_tag(val) {
            IResult::Done(_, tag) => tag,
            _ => panic!("failed to parse paged results value components"),
        }.expect_constructed().expect("paged results components").into_iter();
        let size = match parse_uint(pr_comps.next().expect("element")
                .match_class(TagClass::Universal)
                .and_then(|t| t.match_id(Types::Integer as u64))
                .and_then(|t| t.expect_primitive()).expect("paged results size")
                .as_slice()) {
            IResult::Done(_, size) => size as i32,
            _ => panic!("failed to parse size"),
        };
        let cookie = pr_comps.next().expect("element").expect_primitive().expect("octet string");
        PagedResults { size: size, cookie: cookie }
    }
}
