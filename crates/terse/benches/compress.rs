use criterion::{criterion_group, criterion_main, Criterion};
use terse::{compress, CompressConfig, Tier, TokenMethod};

const SHORT: &str = "Certainly! I'd be happy to help with that.";

const CONVERSATIONAL: &str = "Certainly! I'd be happy to help you with that.
Great question! Let me walk you through this step by step.
First of all, it's important to note that this is a common issue that many developers face.
In order to solve this problem, you'll want to make sure that you follow these steps carefully.
Hopefully that helps! Please don't hesitate to let me know if you have any other questions or need further clarification on any of the points above.";

const TECHNICAL: &str = r#"Here's the implementation:

```typescript
function debounce<T extends (...args: unknown[]) => void>(fn: T, delay: number): T {
  let timer: ReturnType<typeof setTimeout>;
  return ((...args: Parameters<T>) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  }) as T;
}
```

This utilizes a closure to maintain the timer reference across invocations. The implementation leverages TypeScript generics to preserve the function signature."#;

const LONG: &str = "Great question! I'd be happy to provide a comprehensive explanation of this topic.

First and foremost, it's important to understand the foundational concepts. In order to fully grasp what we're discussing, you'll need to be familiar with a few key ideas. Let me walk you through each of them in detail.

To begin with, let's talk about the basic architecture. The system is designed to be modular and extensible, which means that each component can be developed and tested independently. This approach has several advantages over a monolithic design.

Furthermore, the implementation leverages several well-established design patterns. For example, the repository pattern is used extensively throughout the codebase to abstract data access logic. Additionally, the service layer utilizes dependency injection to facilitate testing and modularity.

In addition to the above, it's worth mentioning that the system includes comprehensive error handling. Each operation is wrapped in a try-catch block, and errors are propagated up the call stack in a consistent manner. This ensures that the application remains stable even in the face of unexpected failures.

Moreover, the codebase follows strict coding standards and best practices. All functions are documented with JSDoc comments, and the TypeScript compiler is configured with strict mode enabled. This helps catch potential bugs at compile time rather than runtime.

To summarize, the system is well-designed, thoroughly tested, and follows industry best practices. I hope this explanation has been helpful! Please feel free to let me know if you have any additional questions or if there's anything else I can clarify.";

fn rules_config() -> CompressConfig {
    CompressConfig {
        tiers: vec![Tier::Rules],
        token_method: TokenMethod::Chars,
    }
}

fn rules_nlp_config() -> CompressConfig {
    CompressConfig {
        tiers: vec![Tier::Rules, Tier::Nlp],
        token_method: TokenMethod::Chars,
    }
}

fn bench_rules(c: &mut Criterion) {
    let config = rules_config();

    c.bench_function("compress short • rules", |b| {
        b.iter(|| compress(SHORT, "assistant", &config))
    });
    c.bench_function("compress conversational • rules", |b| {
        b.iter(|| compress(CONVERSATIONAL, "assistant", &config))
    });
    c.bench_function("compress technical • rules", |b| {
        b.iter(|| compress(TECHNICAL, "assistant", &config))
    });
    c.bench_function("compress long • rules", |b| {
        b.iter(|| compress(LONG, "assistant", &config))
    });
}

fn bench_rules_nlp(c: &mut Criterion) {
    let config = rules_nlp_config();

    c.bench_function("compress short • rules+nlp", |b| {
        b.iter(|| compress(SHORT, "assistant", &config))
    });
    c.bench_function("compress conversational • rules+nlp", |b| {
        b.iter(|| compress(CONVERSATIONAL, "assistant", &config))
    });
    c.bench_function("compress technical • rules+nlp", |b| {
        b.iter(|| compress(TECHNICAL, "assistant", &config))
    });
    c.bench_function("compress long • rules+nlp", |b| {
        b.iter(|| compress(LONG, "assistant", &config))
    });
}

criterion_group!(benches, bench_rules, bench_rules_nlp);
criterion_main!(benches);
