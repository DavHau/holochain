use mockall::predicate::eq;

use crate::prelude::*;

#[test]
fn root_is_noop() {
    assert!(Path::from("foo").into_typed(LinkType(0)).exists().unwrap());
    let mut mock = MockHdkT::new();
    mock.expect_create_link().never();
    set_hdk(mock);
    Path::from("foo").into_typed(LinkType(0)).ensure().unwrap();
    assert!(Path::from("foo").into_typed(LinkType(0)).exists().unwrap());
}

#[test]
fn parent_path_committed() {
    let mut mock = MockHdkT::new();
    mock.expect_hash().returning(hash_entry_mock);
    set_hdk(mock);

    let mut mock = MockHdkT::new();
    mock.expect_hash().times(4).returning(hash_entry_mock);
    mock.expect_get_links()
        .once()
        .returning(|_| Ok(vec![vec![]]));
    mock.expect_create_link()
        .once()
        .with(eq(CreateLinkInput {
            base_address: Path::from("foo").path_entry_hash().unwrap().into(),
            target_address: Path::from("foo.bar").path_entry_hash().unwrap().into(),
            link_type: LinkType(0),
            tag: Path::from("bar").make_tag().unwrap(),
            chain_top_ordering: Default::default(),
        }))
        .returning(|_| Ok(HeaderHash::from_raw_36(vec![0; 36])));
    set_hdk(mock);

    Path::from("foo.bar")
        .into_typed(LinkType(0))
        .ensure()
        .unwrap();

    let mut mock = MockHdkT::new();
    mock.expect_hash().returning(hash_entry_mock);
    set_hdk(mock);

    let mut mock = MockHdkT::new();
    mock.expect_hash().times(8).returning(hash_entry_mock);
    mock.expect_get_links()
        .times(2)
        .returning(|_| Ok(vec![vec![]]));
    mock.expect_create_link()
        .once()
        .with(eq(CreateLinkInput {
            base_address: Path::from("foo.bar").path_entry_hash().unwrap().into(),
            target_address: Path::from("foo.bar.baz").path_entry_hash().unwrap().into(),
            link_type: LinkType(0),
            tag: Path::from("baz").make_tag().unwrap(),
            chain_top_ordering: Default::default(),
        }))
        .returning(|_| Ok(HeaderHash::from_raw_36(vec![0; 36])));
    mock.expect_create_link()
        .once()
        .with(eq(CreateLinkInput {
            base_address: Path::from("foo").path_entry_hash().unwrap().into(),
            target_address: Path::from("foo.bar").path_entry_hash().unwrap().into(),
            link_type: LinkType(0),
            tag: Path::from("bar").make_tag().unwrap(),
            chain_top_ordering: Default::default(),
        }))
        .returning(|_| Ok(HeaderHash::from_raw_36(vec![0; 36])));
    set_hdk(mock);

    Path::from("foo.bar.baz")
        .into_typed(LinkType(0))
        .ensure()
        .unwrap();
}

#[test]
fn paths_exists() {
    let mut mock = MockHdkT::new();
    mock.expect_hash().returning(hash_entry_mock);
    set_hdk(mock);

    let mut mock = MockHdkT::new();
    mock.expect_hash().returning(hash_entry_mock);
    mock.expect_get_links().returning(|_| Ok(vec![vec![]]));
    set_hdk(mock);

    assert!(Path::from("foo").into_typed(LinkType(0)).exists().unwrap());
    assert!(Path::from("bar").into_typed(LinkType(0)).exists().unwrap());
    assert!(Path::from("baz").into_typed(LinkType(0)).exists().unwrap());

    assert!(!Path::from("foo.bar")
        .into_typed(LinkType(0))
        .exists()
        .unwrap());
    assert!(!Path::from("foo.bar.baz")
        .into_typed(LinkType(0))
        .exists()
        .unwrap());

    let mut mock = MockHdkT::new();
    mock.expect_hash().returning(hash_entry_mock);
    mock.expect_get_links()
        .once()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: Some(Path::from("bar").make_tag().unwrap()),
        }]))
        .returning(|_| {
            Ok(vec![vec![Link {
                target: Path::from("foo.bar").path_entry_hash().unwrap().into(),
                timestamp: Timestamp::now(),
                tag: Path::from("bar").make_tag().unwrap(),
                create_link_hash: HeaderHash::from_raw_36(vec![0; 36]),
            }]])
        });
    mock.expect_get_links()
        .once()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo.bar").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: Some(Path::from("baz").make_tag().unwrap()),
        }]))
        .returning(|_| {
            Ok(vec![vec![Link {
                target: Path::from("foo.bar.baz").path_entry_hash().unwrap().into(),
                timestamp: Timestamp::now(),
                tag: Path::from("baz").make_tag().unwrap(),
                create_link_hash: HeaderHash::from_raw_36(vec![0; 36]),
            }]])
        });
    set_hdk(mock);

    assert!(Path::from("foo.bar")
        .into_typed(LinkType(0))
        .exists()
        .unwrap());
    assert!(Path::from("foo.bar.baz")
        .into_typed(LinkType(0))
        .exists()
        .unwrap());
}

#[test]
fn children() {
    let mut mock = MockHdkT::new();
    mock.expect_hash().returning(hash_entry_mock);
    set_hdk(mock);

    let foo_bar = Link {
        target: Path::from("foo.bar").path_entry_hash().unwrap().into(),
        timestamp: Timestamp::now(),
        tag: Path::from("bar").make_tag().unwrap(),
        create_link_hash: HeaderHash::from_raw_36(vec![0; 36]),
    };
    let foo_bar2 = Link {
        target: Path::from("foo.bar2").path_entry_hash().unwrap().into(),
        timestamp: Timestamp::now(),
        tag: Path::from("bar2").make_tag().unwrap(),
        create_link_hash: HeaderHash::from_raw_36(vec![0; 36]),
    };
    let foo_bar_baz = Link {
        target: Path::from("foo.bar.baz").path_entry_hash().unwrap().into(),
        timestamp: Timestamp::now(),
        tag: Path::from("baz").make_tag().unwrap(),
        create_link_hash: HeaderHash::from_raw_36(vec![0; 36]),
    };
    let foo_bar2_baz2 = Link {
        target: Path::from("foo.bar2.baz2")
            .path_entry_hash()
            .unwrap()
            .into(),
        timestamp: Timestamp::now(),
        tag: Path::from("baz2").make_tag().unwrap(),
        create_link_hash: HeaderHash::from_raw_36(vec![0; 36]),
    };

    let mut mock = MockHdkT::new();
    mock.expect_hash().returning(hash_entry_mock);
    // foo -[bar]-> foo.bar
    mock.expect_get_links()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: Some(Path::from("bar").make_tag().unwrap()),
        }]))
        .returning({
            let foo_bar = foo_bar.clone();
            move |_| Ok(vec![vec![foo_bar.clone()]])
        });
    // foo -[bar2]-> foo.bar2
    mock.expect_get_links()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: Some(Path::from("bar2").make_tag().unwrap()),
        }]))
        .returning({
            let foo_bar2 = foo_bar2.clone();
            move |_| Ok(vec![vec![foo_bar2.clone()]])
        });
    // foo.bar -[baz]-> foo.bar.baz
    mock.expect_get_links()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo.bar").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: Some(Path::from("baz").make_tag().unwrap()),
        }]))
        .returning({
            let foo_bar_baz = foo_bar_baz.clone();
            move |_| Ok(vec![vec![foo_bar_baz.clone()]])
        });
    // foo.bar2 -[baz2]-> foo.bar2.baz2
    mock.expect_get_links()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo.bar2").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: Some(Path::from("baz2").make_tag().unwrap()),
        }]))
        .returning({
            let foo_bar2_baz2 = foo_bar2_baz2.clone();
            move |_| Ok(vec![vec![foo_bar2_baz2.clone()]])
        });
    // foo -[]-> (foo.bar, foo.bar2)
    mock.expect_get_links()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: None,
        }]))
        .returning(move |_| Ok(vec![vec![foo_bar.clone(), foo_bar2.clone()]]));
    // foo.bar -[]-> foo.bar.baz
    mock.expect_get_links()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo.bar").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: None,
        }]))
        .returning(move |_| Ok(vec![vec![foo_bar_baz.clone()]]));
    // foo.bar2 -[]-> foo.bar2.baz2
    mock.expect_get_links()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo.bar2").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: None,
        }]))
        .returning(move |_| Ok(vec![vec![foo_bar2_baz2.clone()]]));
    // foo.bar.baz -[]-> ()
    mock.expect_get_links()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo.bar.baz").path_entry_hash().unwrap().into(),
            link_type: LinkType(0).into(),
            tag_prefix: None,
        }]))
        .returning(|_| Ok(vec![vec![]]));
    // foo.bar2.baz2 -[]-> ()
    mock.expect_get_links()
        .with(eq(vec![GetLinksInput {
            base_address: Path::from("foo.bar2.baz2")
                .path_entry_hash()
                .unwrap()
                .into(),
            link_type: LinkType(0).into(),
            tag_prefix: None,
        }]))
        .returning(|_| Ok(vec![vec![]]));
    set_hdk(mock);

    assert_eq!(
        Path::from("foo.bar.baz")
            .into_typed(LinkType(0))
            .children_paths()
            .unwrap(),
        vec![]
    );

    assert_eq!(
        Path::from("foo.bar2.baz2")
            .into_typed(LinkType(0))
            .children_paths()
            .unwrap(),
        vec![]
    );

    assert_eq!(
        Path::from("foo.bar")
            .into_typed(LinkType(0))
            .children_paths()
            .unwrap(),
        vec![Path::from("foo.bar.baz").into_typed(LinkType(0))]
    );

    assert_eq!(
        Path::from("foo.bar2")
            .into_typed(LinkType(0))
            .children_paths()
            .unwrap(),
        vec![Path::from("foo.bar2.baz2").into_typed(LinkType(0))]
    );

    assert_eq!(
        Path::from("foo")
            .into_typed(LinkType(0))
            .children_paths()
            .unwrap(),
        vec![
            Path::from("foo.bar").into_typed(LinkType(0)),
            Path::from("foo.bar2").into_typed(LinkType(0)),
        ]
    );
}

fn hash_entry_mock(input: HashInput) -> ExternResult<HashOutput> {
    match input {
        HashInput::Entry(e) => Ok(HashOutput::Entry(EntryHash::with_data_sync(&e))),
        _ => todo!(),
    }
}
