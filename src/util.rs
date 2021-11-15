use std::future::Future;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub async fn repeater<T, F, E, Fut>(interval: Duration, initial: T, work: F) -> E
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut value = initial;
    loop {
        let next_repetition = Instant::now() + interval;
        match work(value).await {
            Ok(v) => value = v,
            Err(err) => return err,
        }
        if Instant::now() < next_repetition {
            sleep(next_repetition - Instant::now()).await;
        }
    }
}

#[tokio::test]
async fn test_repeater() {
    let start = Instant::now();
    let res = repeater(Duration::from_millis(20), 0, |mut counter| async move {
        counter += 1;
        if counter == 5 {
            return Err(counter);
        }
        Ok(counter)
    })
    .await;
    assert_eq!(res, 5);
    assert!(Instant::now() - start > Duration::from_millis(80));
    assert!(Instant::now() - start < Duration::from_millis(150));
}

#[tokio::test]
async fn test_repeater_slow_worker() {
    let start = Instant::now();
    repeater(Duration::from_millis(5), 0, |mut counter| async move {
        sleep(Duration::from_millis(20)).await;
        counter += 1;
        if counter == 5 {
            return Err(counter);
        }
        Ok(counter)
    })
    .await;
    assert!(Instant::now() - start > Duration::from_millis(80));
}
