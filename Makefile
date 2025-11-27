# compile everything aot at once
test/%.all: test/%.snek src/main.rs runtime/start.rs
	cargo run --target x86_64-apple-darwin -- -c $< test/$*.s
	nasm -f macho64 test/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc --target x86_64-apple-darwin -L runtime/ runtime/start.rs -o test/$*.run


test/%.allt: test/%.snek src/main.rs runtime/start.rs
	cargo run --target x86_64-apple-darwin -- -tc $< test/$*.s
	nasm -f macho64 test/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc --target x86_64-apple-darwin -L runtime/ runtime/start.rs -o test/$*.run


# Compile .snek to .s (AOT compilation)
test/%.s: test/%.snek src/main.rs
	cargo run --target x86_64-apple-darwin -- -c $< test/$*.s

test/%.ts: test/%.snek src/main.rs
	cargo run --target x86_64-apple-darwin -- -tc $< test/$*.s

test/%.t: test/%.snek src/main.rs
	cargo run --target x86_64-apple-darwin -- -t $< test/$*.s

# Compile .s to executable
test/%.run: test/%.s runtime/start.rs
	nasm -f macho64 test/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc --target x86_64-apple-darwin -L runtime/ runtime/start.rs -o test/$*.run

# JIT execute only (no assembly file generated)
test/%.jit: test/%.snek src/main.rs
	cargo run --target x86_64-apple-darwin -- -e test/$*.snek $(filter-out $@,$(MAKECMDGOALS))

test/%.jitt: test/%.snek src/main.rs
	cargo run --target x86_64-apple-darwin -- -te test/$*.snek $(filter-out $@,$(MAKECMDGOALS))

# Both JIT execute and generate assembly (debugging)
test/%.debug: test/%.snek src/main.rs
	cargo run --target x86_64-apple-darwin -- -g $< test/$*.s $(filter-out $@,$(MAKECMDGOALS))

test/%.debugt: test/%.snek src/main.rs
	cargo run --target x86_64-apple-darwin -- -tg $< test/$*.s $(filter-out $@,$(MAKECMDGOALS))

clean:
	rm -f test/*.s test/*.run runtime/*.o runtime/*.a

# JIT execute only (no assembly file generated)
repl: 
	rlwrap cargo run --target x86_64-apple-darwin -- -i 

replt: 
	rlwrap cargo run --target x86_64-apple-darwin -- -ti 

# Convenience targets
.PHONY: clean repl