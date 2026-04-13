pub const TEMPLATE_STYLES: &str = r#"
:root {
    --leptoaster-width: 320px;
    --leptoaster-max-width: 80vw;
    --leptoaster-z-index: 9999;

    --leptoaster-font-family: 'Inter', ui-sans-serif, system-ui, -apple-system, sans-serif;
    --leptoaster-font-size: 13px;
    --leptoaster-line-height: 18px;
    --leptoaster-font-weight: 500;

    --leptoaster-progress-height: 2px;

    --leptoaster-info-background-color: var(--background, #ffffff);
    --leptoaster-info-border-color: var(--border, #e5e5e5);
    --leptoaster-info-text-color: var(--foreground, #222222);

    --leptoaster-success-background-color: oklch(0.55 0.18 160);
    --leptoaster-success-border-color: oklch(0.45 0.16 160);
    --leptoaster-success-text-color: #ffffff;

    --leptoaster-warn-background-color: oklch(0.72 0.15 80);
    --leptoaster-warn-border-color: oklch(0.62 0.13 80);
    --leptoaster-warn-text-color: oklch(0.12 0.005 250);

    --leptoaster-error-background-color: oklch(0.55 0.22 25);
    --leptoaster-error-border-color: oklch(0.45 0.2 25);
    --leptoaster-error-text-color: #ffffff;
}

.dark {
    --leptoaster-info-background-color: var(--card, oklch(0.12 0.003 250));
    --leptoaster-info-border-color: var(--border, oklch(1 0 0 / 6%));
    --leptoaster-info-text-color: var(--foreground, oklch(0.97 0.002 250));

    --leptoaster-success-background-color: oklch(0.73 0.14 160);
    --leptoaster-success-border-color: oklch(0.63 0.12 160);

    --leptoaster-warn-background-color: oklch(0.82 0.15 80);
    --leptoaster-warn-border-color: oklch(0.72 0.13 80);
    --leptoaster-warn-text-color: oklch(0.08 0.005 250);

    --leptoaster-error-background-color: oklch(0.68 0.2 25);
    --leptoaster-error-border-color: oklch(0.58 0.18 25);
}

.leptoaster-stack-container-bottom:hover > div,
.leptoaster-stack-container-top:hover > div {
    opacity: 1 !important;
    transform: translateY(0) scaleX(1) !important;
    transition-delay: 0s !important;
}

.leptoaster-stack-container-bottom > div:nth-last-child(1),
.leptoaster-stack-container-top > div:nth-child(1) {
    z-index: 9999;
}

.leptoaster-stack-container-bottom > div:nth-last-child(2),
.leptoaster-stack-container-top > div:nth-child(2) {
    z-index: 9998;
}

.leptoaster-stack-container-bottom > div:nth-last-child(2) {
    transform: translateY(62px) scaleX(0.98);
}

.leptoaster-stack-container-top > div:nth-child(2) {
    transform: translateY(-62px) scaleX(0.98);
}

.leptoaster-stack-container-bottom > div:nth-last-child(3),
.leptoaster-stack-container-top > div:nth-child(3) {
    z-index: 9997;
}

.leptoaster-stack-container-bottom > div:nth-last-child(3) {
    transform: translateY(124px) scaleX(0.96);
}

.leptoaster-stack-container-top > div:nth-child(3) {
    transform: translateY(-124px) scaleX(0.96);
}

.leptoaster-stack-container-bottom > div:nth-last-child(4),
.leptoaster-stack-container-top > div:nth-child(4) {
    z-index: 9996;
}

.leptoaster-stack-container-bottom > div:nth-last-child(4) {
    transform: translateY(186px) scaleX(0.94);
}

.leptoaster-stack-container-top > div:nth-child(4) {
    transform: translateY(-186px) scaleX(0.94);
}

.leptoaster-stack-container-bottom > div:nth-last-child(5),
.leptoaster-stack-container-top > div:nth-child(5) {
    z-index: 9995;
}

.leptoaster-stack-container-bottom > div:nth-last-child(5) {
    transform: translateY(248px) scaleX(0.92);
}

.leptoaster-stack-container-top > div:nth-child(5) {
    transform: translateY(-248px) scaleX(0.92);
}

.leptoaster-stack-container-bottom > div:nth-last-child(n+6),
.leptoaster-stack-container-top > div:nth-child(n+6) {
    opacity: 0;
}

@keyframes leptoaster-slide-in-left {
    from { left: calc((var(--leptoaster-width) + 12px * 2) * -1) }
    to { left: 0 }
}

@keyframes leptoaster-slide-out-left {
    from { left: 0 }
    to { left: calc((var(--leptoaster-width) + 12px * 2) * -1) }
}

@keyframes leptoaster-slide-in-right {
    from { right: calc((var(--leptoaster-width) + 12px * 2) * -1) }
    to { right: 0 }
}

@keyframes leptoaster-slide-out-right {
    from { right: 0 }
    to { right: calc((var(--leptoaster-width) + 12px * 2) * -1) }
}

@keyframes leptoaster-progress {
    from { width: 100%; }
    to { width: 0; }
}
"#;
