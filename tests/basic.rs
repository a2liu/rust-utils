use aliu::*;

#[test]
fn test_with_bucket_list() {
    let capacity = 128;
    let iterations = 4;
    let data = [12u64, 12, 31, 4123];

    let bucket_list = &BucketList::with_capacity(capacity);

    for _ in 0..iterations {
        let mut pod = Pod::<u64, _>::with_allocator(bucket_list);
        pod.reserve(data.len());

        for i in data {
            pod.push(i);
        }

        println!("len={}, capa={}: {:?}", pod.len(), pod.capacity(), pod);
    }

    let used = data.len() * 8 * iterations;
    assert_eq!(bucket_list.total_used(), used);
    assert_eq!(bucket_list.total_capacity(), 128);

    bucket_list.new(1u64);

    assert_eq!(bucket_list.total_used(), used + 8);
    assert_eq!(
        bucket_list.total_capacity(),
        128 + BucketList::DEFAULT_BUCKET_SIZE
    );
}

#[test]
fn test_basics() {
    let a = r(0usize, 5);
    let b = r(0u32, 5u32);
    let c = 0usize;
    let d = 0u32;

    let data = pod![1, 2, 3, 4, 5, 6, 7];

    assert_eq!(&[1, 2, 3, 4, 5], &data[a]);
    assert_eq!(&[1, 2, 3, 4, 5], &data[b]);
    assert_eq!(&1, &data[c]);
    assert_eq!(&1, &data[d]);
}
