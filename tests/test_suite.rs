use draft::test_draft;

#[cfg(feature = "perfect_precision")]
test_draft!("tests/suite/tests/draft4/");
#[cfg(not(feature = "perfect_precision"))]
test_draft!("tests/suite/tests/draft4/", {"optional_bignum_0_0", "optional_bignum_2_0"});

test_draft!("tests/suite/tests/draft6/");

test_draft!("tests/suite/tests/draft7/");
