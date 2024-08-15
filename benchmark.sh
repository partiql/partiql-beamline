cargo build --release;
echo "GEN TEXT"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format text  > test-o.txt.ion;
echo ""
echo ""
echo "GEN ION"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-pretty  > test-o.ip.ion;
echo ""
echo ""
echo "GEN IONC"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion  > test-o.c.ion;

echo ""
echo ""
echo "GEN FILTERED TEXT"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format text  > test-o.txt.f.ion;

echo ""
echo ""
echo "GEN FILTERED ION"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-pretty  > test-o.ip.f.ion;

echo ""
echo ""
echo "GEN FILTERED IONC"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion  > test-o.c.f.ion;


