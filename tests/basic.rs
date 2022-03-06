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
