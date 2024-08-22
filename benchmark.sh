SAMPLES=500000

cargo build --release;

echo "GEN TEXT"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count $SAMPLES \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format text  > test-o.txt.ion;
du -h test-o.txt.ion
zstd test-o.txt.ion -f -12  -o test-o.txt.ion.zst
du -h test-o.txt.ion.zst
echo ""
echo ""
echo "GEN ION"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count $SAMPLES \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-pretty  > test-o.ip.ion;
du -h test-o.ip.ion
zstd test-o.ip.ion -f -12 -o test-o.ip.ion.zst
du -h test-o.ip.ion.zst
echo ""
echo ""
echo "GEN IONC"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count $SAMPLES \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion  > test-o.c.ion;
du -h test-o.c.ion
zstd test-o.c.ion -f -12 -o test-o.c.ion.zst
du -h test-o.c.ion.zst
echo ""
echo ""
echo "GEN IONB"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count $SAMPLES \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-binary  > test-o.b.i0n;
du -h test-o.b.i0n
zstd test-o.b.i0n -f -12 -o test-o.b.i0n.zst
du -h test-o.b.i0n.zst

echo ""
echo ""
echo "GEN FILTERED TEXT"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count $SAMPLES \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format text  > test-o.txt.f.ion;
du -h test-o.txt.f.ion

echo ""
echo ""
echo "GEN FILTERED ION"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count $SAMPLES \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-pretty  > test-o.ip.f.ion;
du -h test-o.ip.f.ion

echo ""
echo ""
echo "GEN FILTERED IONC"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count $SAMPLES \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion  > test-o.c.f.ion;
du -h test-o.c.f.ion

echo ""
echo ""
echo "GEN FILTERED IONB"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count $SAMPLES \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-binary  > test-o.b.f.i0n;
du -h test-o.b.f.i0n




