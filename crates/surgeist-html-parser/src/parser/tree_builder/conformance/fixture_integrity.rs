//! Integrity pin for the retained html5lib-tests tree-construction files.
//!
//! The files are verbatim copies of html5lib-tests revision
//! `f994590f528ac8b6073665791ddb1ed85c66dfb2` (see the crate NOTICE). Each row
//! pins one file's case count and SHA-256, both taken from the bytes on disk
//! independently of the `.dat` decoder: the count is the number of `#data`
//! section markers (`grep -a -c '^#data'`) and the digest is `shasum -a 256`.
//! A changed, truncated or substituted file therefore fails here by name,
//! instead of shifting an aggregate total.

/// `(file name, case count, SHA-256 of the file bytes)` in `DATA` order.
pub(super) const TREE_CONSTRUCTION: [(&str, usize, &str); 56] = [
    (
        "adoption02.dat",
        2,
        "6ca94baf47ef9b6756657f40b15a12ac758802faa5003a7659652b8664e1e7d8",
    ),
    (
        "namespace-sensitivity.dat",
        1,
        "318fbc9926eddf5863524f1503c711ddaea93b55bcbd1eaab71b7e354ddfeb09",
    ),
    (
        "quirks01.dat",
        4,
        "b6717cc15d4ed573ccf6755bc9d52073e675b4d09f23bde07961e27592b9717c",
    ),
    (
        "tests14.dat",
        7,
        "d151b2426f38de40a5d4ae726e2a56dbc7742a9b1c95299d60c1e2e0fdad1f98",
    ),
    (
        "tests24.dat",
        8,
        "fdd5c21f60f42235ded224a03e7182d289088328f460525adaf3c14e772a24ac",
    ),
    (
        "tables01.dat",
        19,
        "bfd4a53246e3acc527c8bb214cc743082e19260e72365001aad3f1d8f4bd08dc",
    ),
    (
        "comments01.dat",
        16,
        "79a7a42412b6dc48fa0c779dd8ba48d4d50033e7e9e417619288eec1e59d8db2",
    ),
    (
        "template.dat",
        112,
        "74bc8308e673589d81b63eb71411a6f8ecea6fd6af649b5802c22615b7102a03",
    ),
    (
        "adoption01.dat",
        18,
        "b2aba05bd1d832f73a0c6103b3c8b151b283bab7c56887274d81f3a062c4963e",
    ),
    (
        "tests1.dat",
        112,
        "8438d7948b366c77821cff1fc71826a8090759a78ac8a691a5c82a51fc7e5d57",
    ),
    (
        "tests2.dat",
        63,
        "9cf76b5f4890065c04fc82ae828379a55b85cbe76f584fb1ef23dcef0a77b86b",
    ),
    (
        "tests3.dat",
        24,
        "c4b4d8e0ea3d978c49d1e6a985d427164858b71ec980153c4526ed0c398f43b9",
    ),
    (
        "tests4.dat",
        9,
        "e6003a52e1cbffc361eca7c739cdd4459074ac8c59766d1eb171b9c9eb42f517",
    ),
    (
        "tests5.dat",
        17,
        "bf80b927082290541781844906abdcb72f091488f7a11e8b7d9ac00076dd4fce",
    ),
    (
        "tests6.dat",
        52,
        "be16c74d2a9862283262968439c95b9bfce6182ea61055a3c886c28d14f3e01a",
    ),
    (
        "tests7.dat",
        34,
        "aeb9569589c809b1a563c0b163173e8cf980a6a4028cb7c210f19c181b56999a",
    ),
    (
        "tests8.dat",
        10,
        "f3c8b1baece162e7e8c394540cdf9057266bb738bb0ad539a9d9b2f9d365f162",
    ),
    (
        "tests9.dat",
        27,
        "857820f088506a6d20ea6acefbf19c0a6c4de87a39b24988878066397516b36f",
    ),
    (
        "tests10.dat",
        54,
        "2d2624a819c323661e396d864ac23440053127b5ea7adb44a5904f5ceee5fa64",
    ),
    (
        "tests11.dat",
        13,
        "276190e2a7b97e8fcf3bd863a4b4b5346b555a8336c00143cb1d0e8956b94a07",
    ),
    (
        "tests12.dat",
        2,
        "e6c506cea74979a0d6ca47f6b175c7b6177d6e57db708920a89694e31dfd8a42",
    ),
    (
        "tests15.dat",
        14,
        "ef784ece74cbd760da3a6947aaf4478810246b73c753f25a04be4d11dd806b2d",
    ),
    (
        "tests16.dat",
        197,
        "3350be682713afc1f6dad37059f2551709497a3643c38c26e1fe36fd07d23745",
    ),
    (
        "tests17.dat",
        13,
        "0567680775f58a5b2ad24e234f41d53f68fb2fc3ef7809bebaec0920e2f13c89",
    ),
    (
        "tests18.dat",
        36,
        "5d0019ae43bb4e0b0da9f2e1d57ac0618a607bd8a1324b163ec7a23a1dc120f3",
    ),
    (
        "tests19.dat",
        103,
        "a9316b1eb4d2821a18e2c840394c6218bfcdb598857b92631490d9c3c1840ce4",
    ),
    (
        "tests20.dat",
        64,
        "07f7661690c4cd7cc0bbb0f1b9c1e1d65135e07c4dde8bbf106692f687e7d33d",
    ),
    (
        "tests21.dat",
        23,
        "b1a67420c79a5131002fefc987084ffb6b6094a3a74b272c6f545a47df06452c",
    ),
    (
        "tests22.dat",
        5,
        "78488328181d0f82f34b1a5e9e456ff7c713b3ff86dab6ace1530f6f07d5370d",
    ),
    (
        "tests23.dat",
        5,
        "2e4752ff4ef898e4a0cf9a450e481440095163d45075a662846a062552afe148",
    ),
    (
        "tests25.dat",
        26,
        "f2e08fda6d15a08faf9ff0001ec38560d942069cee6b4e264aeedbbacde7da8d",
    ),
    (
        "tests26.dat",
        20,
        "d55d24dfca2444fba759d59346cbab21f7e70340dbb14e2c7af8da3d44abd4ab",
    ),
    (
        "webkit01.dat",
        52,
        "063ca232535a792fa238ae769eeb0ddcc9bd2ee961d3da327ca134612d35d93c",
    ),
    (
        "webkit02.dat",
        46,
        "9305e1089d23f18f15a392d1290cb2a438c952d2bd8e5239802f111ad8f3771e",
    ),
    (
        "entities01.dat",
        75,
        "b73605caac5aed5656184ab8db3f08edff5457ac3186cb04afa28384bca955e3",
    ),
    (
        "entities02.dat",
        26,
        "f4e0cd461204b0184709be040c00811b776fbde1a6d002bb1c75d02056d9e54a",
    ),
    (
        "ruby.dat",
        21,
        "5ae76ac4570d40e6648066798dd3ff729231bec98d3e010fdd67e0257f23a742",
    ),
    (
        "foreign-fragment.dat",
        66,
        "73e1785753c66420c067e5b29f89b17e7ebe1079687a512040ef19c6d42ac2c2",
    ),
    (
        "noscript01.dat",
        18,
        "e82449304a6371c14ed490384b6d814b7b1eeb1022dbc5caa2045fc8400655d7",
    ),
    (
        "tests_innerHTML_1.dat",
        81,
        "932393c64d63ba14d81edadefc7d441c5e0b06370ebec31488c5dc5840282ba6",
    ),
    (
        "blocks.dat",
        48,
        "e3b7da1b57a4ec6991443dfc7ab3270f41d0f5959092a49be3e0edca0dae2104",
    ),
    (
        "doctype01.dat",
        37,
        "f3a286c09d729eeed9aa63e0aec13ab12336af04590c169543e6d1e1ef4723b2",
    ),
    (
        "domjs-unsafe.dat",
        49,
        "eef4fb719e027ffaadfb854787d172893850fc561c29efe72d90c5bfa3c8b7ac",
    ),
    (
        "html5test-com.dat",
        24,
        "71d2ecbd0947599e82c876ce0300ecfc3a8cafd9d1736b33913f6a6152993bde",
    ),
    (
        "inbody01.dat",
        4,
        "cb722f2853ec9613b71ea68e8bb26f474cc450b64612930591ab2656406222fe",
    ),
    (
        "isindex.dat",
        4,
        "d152de773e276a07a1cfc91e93f01f3ce52c447c110192f2828dab51be2721b6",
    ),
    (
        "main-element.dat",
        3,
        "d56e382994e1a5228ddb2ea49a5ff51ab68487f9a8f79ad34bcad05d40dc1e8f",
    ),
    (
        "math.dat",
        8,
        "3c2ecc07272c175676ecfafa0cf6e18e74c3293075703702a4b7929fcb0d07bd",
    ),
    (
        "menuitem-element.dat",
        20,
        "08e5e25f38bbc181c840ce5c5203b2566dc03129aeaf105ca2311645d40aae4d",
    ),
    (
        "pending-spec-changes.dat",
        3,
        "a6b7c4ecccabe70de2f24184245e4a73a8d12ed44ee658c3d5d25cce04c9d27f",
    ),
    (
        "pending-spec-changes-plain-text-unsafe.dat",
        1,
        "f45151f8dc7a4fe1a4b36710cf33606ff43cc98a42f1d2f085a77680683b0c99",
    ),
    (
        "plain-text-unsafe.dat",
        33,
        "1aef5e36b7ea04bb1d70684b33529a183938b7604283c562c698d6dad4e3878d",
    ),
    (
        "scriptdata01.dat",
        26,
        "e32ea3adc3d68c90e62ea50b3a41bf80369d1491958532bf00787b3509667322",
    ),
    (
        "search-element.dat",
        3,
        "30be0e9e8cbeea825e0323a7a3a518ab88f44fcada9b4f82b0cc7bbeb9334b76",
    ),
    (
        "svg.dat",
        8,
        "4c819b8dbdfbd98cfbce9a535a304b0a16597eb29ccc04077131a62810f309cd",
    ),
    (
        "tricky01.dat",
        9,
        "3fb6d24c5e371860d096ef07f5fff38be3ceaa85f1239bee35cce548b596198a",
    ),
];

/// FIPS 180-4 SHA-256, as lowercase hexadecimal. The crate has no hashing
/// dependency; this test-only implementation is checked against the standard's
/// published vectors below and against `shasum` through the pinned digests.
pub(super) fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&(u64::try_from(data.len()).unwrap() * 8).to_be_bytes());
    let (blocks, _) = message.as_chunks::<64>();
    for block in blocks {
        let mut w = [0u32; 64];
        let (words, _) = block.as_chunks::<4>();
        for (word, bytes) in w.iter_mut().zip(words) {
            *word = u32::from_be_bytes(*bytes);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

#[test]
fn sha256_matches_the_published_fips_180_vectors() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn retained_tree_construction_files_match_their_pinned_digests_in_data_order() {
    assert_eq!(super::DATA.len(), TREE_CONSTRUCTION.len());
    for ((name, raw), (pinned, _, digest)) in super::DATA.iter().zip(TREE_CONSTRUCTION) {
        assert_eq!(
            *name, pinned,
            "DATA and the integrity pin list the same files"
        );
        assert_eq!(sha256_hex(raw.as_bytes()), digest, "{name}");
    }
}
