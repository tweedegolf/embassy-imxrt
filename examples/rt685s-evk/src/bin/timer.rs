#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::clocks::config::PoweredClock;
use embassy_imxrt::timer::{CTimerSel, CaptureChEdge, CaptureTimer, CountingTimer, TimerChannelNum, TimerConfig};
use embassy_imxrt::{bind_interrupts, peripherals, timer};
use embassy_time::Timer;
use {defmt_rtt as _, embassy_imxrt_examples as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CTIMER0 => timer::InterruptHandler<peripherals::CTIMER0>;
    CTIMER1 => timer::InterruptHandler<peripherals::CTIMER1>;
    CTIMER4 => timer::InterruptHandler<peripherals::CTIMER4>;
});

// Monitor task is created to demonstrate difference between Async and Blocking timer behavior
#[embassy_executor::task]
async fn monitor_task() {
    loop {
        info!("Secondary task running");
        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut cfg = embassy_imxrt::config::Config::default();
    // Ensure the SFRO is enabled
    cfg.clocks.enable_16m_irc = Some(PoweredClock::AlwaysEnabled);

    let mut p = embassy_imxrt::init(cfg);

    // spawner.spawn(monitor_task()).unwrap();

    // for _ in 0..100_000 {
    //     info!("Hello :)");
    //     cortex_m::asm::delay(10_000_000);
    // }

    let mut tmr1 = CountingTimer::new_blocking(
        p.CTIMER0.reborrow(),
        TimerChannelNum::Channel0,
        TimerConfig {
            source: CTimerSel::FfroClk,
            powered: PoweredClock::AlwaysEnabled,
        },
    );

    let mut tmr2 = CountingTimer::new_async(
        p.CTIMER1.reborrow(),
        TimerChannelNum::Channel0,
        TimerConfig {
            source: CTimerSel::FfroClk,
            powered: PoweredClock::AlwaysEnabled,
        },
        Irqs,
    );

    tmr1.wait_us(3000000); // 3 seconds wait
    info!("First Counting timer expired");

    tmr2.wait_us(5000000).await; //  5 seconds wait
    info!("Second Counting timer expired");

    {
        let mut cap_async_tmr = CaptureTimer::new_async(
            p.CTIMER4.reborrow(),
            TimerChannelNum::Channel0,
            TimerConfig {
                source: CTimerSel::FfroClk,
                powered: PoweredClock::AlwaysEnabled,
            },
            p.PIO0_5.reborrow(),
            Irqs,
        );
        let event_time_us = cap_async_tmr.capture_cycle_time_us(CaptureChEdge::Rising).await;
        info!("Capture timer expired, time between two capture = {} us", event_time_us);

        drop(cap_async_tmr);

        let mut cap_async_tmr = CaptureTimer::new_async(
            p.CTIMER4.reborrow(),
            TimerChannelNum::Channel0,
            TimerConfig {
                source: CTimerSel::FfroClk,
                powered: PoweredClock::AlwaysEnabled,
            },
            p.PIO0_5.reborrow(),
            Irqs,
        );
        let event_time_us = cap_async_tmr.capture_cycle_time_us(CaptureChEdge::Rising).await;
        info!("Capture timer expired, time between two capture = {} us", event_time_us);
    }

    loop {
        // This code is showing how to use the timer in a periodic fashion
        tmr2.wait_us(5000000).await;
        info!("Primary task running");
    }
}
