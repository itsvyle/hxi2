import "./dvd.scss";
export interface DvdOverlayConfig {
    speed?: number;
    imageWidth?: number;
    imageHeight?: number;
}

export class DvdScreensaver {
    private overlayElement: HTMLElement;
    private logoElement: HTMLImageElement;
    private config: Required<DvdOverlayConfig>;

    private x: number = 50;
    private y: number = 50;
    private dx: number = 1;
    private dy: number = 1;
    private animationFrameId: number | null = null;

    private defaultSpeed: number = 2;
    private defaultImageWidth: number = 150;

    private displayBox: {
        tl: number;
        tr: number;
        bl: number;
        br: number;
    } | null = null;

    // When over transparency element, the logo will be transparent
    constructor(
        overlayId: string = "dvd-overlay",
        transparencyElement?: HTMLElement,
    ) {
        const overlay = document.getElementById(overlayId);
        const logo = overlay.getElementsByTagName("img")[0] as HTMLImageElement;

        if (!overlay || !logo) {
            throw new Error("Overlay or Logo element not found in the DOM.");
        }
        this.overlayElement = overlay;
        this.logoElement = logo;

        this.config = {
            speed: this.defaultSpeed,
            imageWidth: this.defaultImageWidth,
            imageHeight: 0,
        };

        if (transparencyElement) {
            const ref = () => {
                const rect = transparencyElement.getBoundingClientRect();
                this.displayBox = {
                    tl: rect.top,
                    tr: rect.top + rect.width,
                    bl: rect.bottom,
                    br: rect.bottom + rect.width,
                };
            };
            ref();
            const resizeObserver = new ResizeObserver((entries) => {
                ref();
            });
            resizeObserver.observe(transparencyElement);
            window.addEventListener("resize", ref);
        }
    }

    public show(userConfig: DvdOverlayConfig): void {
        this.config = {
            speed: userConfig.speed ?? this.defaultSpeed,
            imageWidth: userConfig.imageWidth ?? this.defaultImageWidth,
            imageHeight: userConfig.imageHeight ?? 0,
        };

        this.logoElement.style.width = `${this.config.imageWidth}px`;

        if (this.config.imageHeight && this.config.imageHeight > 0) {
            this.logoElement.style.height = `${this.config.imageHeight}px`;
        } else {
            this.logoElement.style.height = "auto";
        }

        this.x =
            Math.random() * (window.innerWidth - this.logoElement.offsetWidth);
        this.y =
            Math.random() *
            (window.innerHeight - this.logoElement.offsetHeight);
        this.dx = (Math.random() < 0.5 ? 1 : -1) * this.config.speed;
        this.dy = (Math.random() < 0.5 ? 1 : -1) * this.config.speed;

        this.overlayElement.classList.add("active");

        this.startAnimation();
    }

    private startAnimation(): void {
        if (this.animationFrameId) {
            cancelAnimationFrame(this.animationFrameId);
        }
        this.animate();
    }

    public hide(): void {
        this.overlayElement.classList.remove("active");
        if (this.animationFrameId) {
            cancelAnimationFrame(this.animationFrameId);
            this.animationFrameId = null;
        }
    }

    public toggle(userConfig: DvdOverlayConfig): void {
        if (this.overlayElement.classList.contains("active")) {
            this.hide();
        } else {
            this.show(userConfig);
        }
    }

    private animate = (): void => {
        const logoWidth = this.logoElement.offsetWidth;
        const logoHeight = this.logoElement.offsetHeight;
        const screenWidth = window.innerWidth;
        const screenHeight = window.innerHeight;

        this.x += this.dx;
        this.y += this.dy;

        let bounced = false;

        if (this.x + logoWidth >= screenWidth) {
            this.dx = -Math.abs(this.dx);
            this.x = screenWidth - logoWidth;
            bounced = true;
        } else if (this.x <= 0) {
            this.dx = Math.abs(this.dx);
            this.x = 0;
            bounced = true;
        }

        if (this.y + logoHeight >= screenHeight) {
            this.dy = -Math.abs(this.dy);
            this.y = screenHeight - logoHeight;
            bounced = true;
        } else if (this.y <= 0) {
            this.dy = Math.abs(this.dy);
            this.y = 0;
            bounced = true;
        }

        // Optional: Change hue on bounce for fun
        // if (bounced) {
        //    this.logoElement.style.filter = `hue-rotate(${Math.random() * 360}deg)`;
        // }

        if (this.displayBox) {
            const { tl, tr, bl, br } = this.displayBox;
            if (
                this.y + logoHeight >= tl &&
                this.y <= bl &&
                this.x + logoWidth >= tr &&
                this.x <= br
            ) {
                this.logoElement.style.opacity = "0.5";
            } else {
                this.logoElement.style.opacity = "1";
            }
        } else {
            this.logoElement.style.opacity = "1";
        }

        this.logoElement.style.left = `${this.x}px`;
        this.logoElement.style.top = `${this.y}px`;

        this.animationFrameId = requestAnimationFrame(this.animate);
    };
}
