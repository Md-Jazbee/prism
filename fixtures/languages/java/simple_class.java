/**
 * Minimal Java fixture for T1 golden conformance.
 *
 * Covers: class def, import, extends, method defs, same-file call, unresolved call.
 */
import java.util.List;

public class Child extends Base {
    public void helper(int x) {
        return;
    }

    public void run() {
        helper(1);
        missing_fn();
    }
}
