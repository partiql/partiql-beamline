cargo build --release;
echo "GEN TEXT-O"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format text  > test-o.txt.ion;
echo "GEN TEXT-R"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data-refactor \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format text  > test-r.txt.ion;
echo ""
echo ""
echo "GEN ION-O"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-pretty  > test-o.ip.ion;
echo "GEN ION-R"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data-refactor \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-pretty  > test-r.ip.ion;
echo ""
echo ""
echo "GEN IONC-O"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion  > test-o.c.ion;
echo "GEN IONC-R"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data-refactor \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion  > test-r.c.ion;
echo ""
echo ""
echo "DIFF TEXT"
diff -s -q test-o.txt.ion test-r.txt.ion
echo ""
echo ""
echo "DIFF ION PRETTY"
diff -s -q test-o.ip.ion test-r.ip.ion
echo ""
echo ""
echo "DIFF ION COMPACT"
diff -s -q test-o.c.ion test-r.c.ion

echo ""
echo ""
echo "TESTING FILTERED"
echo ""
echo ""

echo "GEN FILTERED TEXT-O"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format text  > test-o.txt.f.ion;
echo "GEN FILTERED TEXT-R"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data-refactor \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format text  > test-r.txt.f.ion;
echo ""
echo ""
echo "GEN FILTERED ION-O"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-pretty  > test-o.ip.f.ion;
echo "GEN FILTERED ION-R"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data-refactor \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion-pretty  > test-r.ip.f.ion;
echo ""
echo ""
echo "GEN FILTERED IONC-O"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion  > test-o.c.f.ion;
echo "GEN IONC-R"
/usr/bin/time -l ./target/release/partiql-beamline-cli gen data-refactor \
                        --dataset service \
                        --seed 5599165213374806994 --start-iso "2024-01-20T20:51:02.000000000Z" --sample-count 500000 \
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
                        --output-format ion  > test-r.c.f.ion;
echo ""
echo ""
echo "DIFF FILTERED TEXT"
diff -s -q test-o.txt.f.ion test-r.txt.f.ion
echo ""
echo ""
echo "DIFF FILTERED ION PRETTY"
diff -s -q test-o.ip.f.ion test-r.ip.f.ion
echo ""
echo ""
echo "DIFF FILTERED ION COMPACT"
diff -s -q test-o.c.f.ion test-r.c.f.ion
