class EventIterator implements AsyncIterable<Event>, AsyncIterator<Event> {
    private eventSource: EventSource;
    private queue: Event[] = [];
    private resolveQueue: ((value: IteratorResult<Event>) => void) | null = null;

    constructor(url: string) {
        this.eventSource = new EventSource(url);
        this.eventSource.onmessage = (event) => this.handleEvent(event);
    }

    [Symbol.asyncIterator](): AsyncIterableIterator<Event> {
        return this;
    }

    async next(): Promise<IteratorResult<Event>> {
        if (this.queue.length > 0) {
            return Promise.resolve({ value: this.queue.shift()!, done: false });
        }

        return new Promise((resolve) => {
            this.resolveQueue = resolve;
        });
    }

    private handleEvent(event: MessageEvent): void {
        const newEvent = this.parseEvent(event.data);

        if (this.resolveQueue) {
            this.resolveQueue({ value: newEvent, done: false });
            this.resolveQueue = null;
        } else {
            this.queue.push(newEvent);
        }
    }

    private parseEvent(data: string): Event {
        // Parse the event data from the backend
        return JSON.parse(data);
    }

    close(): void {
        this.eventSource.close();
    }
}