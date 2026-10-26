import { attributes } from "../../css/shared.js";
import { DEFAULT_TEXTS } from "../../default-texts.js";
/**
 * The class on the host, that is, the region the messages are in.
 *
 * Until 0.28.0 this was the class on each message, and the region was styled
 * by its element name. Registered under another name, with
 * `defineLtToast("my-toast")`, it then lost its position.
 */
export const TOAST_CLASS = "lt-toast";
export const TOAST_MESSAGE_CLASS = "lt-toast__message";
export const TOAST_CLOSE_CLASS = "lt-toast__close";
/**
 * The attributes for the toast region.
 *
 * The region is `role="status"` and not `role="alert"`. A message that comes
 * because something went well should not interrupt what the screen reader is
 * doing.
 *
 * `data-ignore-morph` is not optional. `<lt-toast>` creates and removes its
 * message elements inside the region, and without the attribute the Datastar
 * morph tears them away at the next patch, because the server's version of
 * the region is empty. The attribute must be in the HTML the server sends:
 * the component cannot set it on itself, since the morph reads it from the
 * server's node.
 */
export const toast = ({ label = DEFAULT_TEXTS.toastRegion, closeLabel, } = {}) => ({
    host: attributes({
        class: TOAST_CLASS,
        role: "status",
        "aria-live": "polite",
        // `status` is atomic by default, and then the whole stack is announced
        // again for every message that is added.
        "aria-atomic": "false",
        "aria-label": label,
        "close-label": closeLabel,
        "data-ignore-morph": "",
    }),
    message: attributes({ class: TOAST_MESSAGE_CLASS }),
    close: attributes({ class: TOAST_CLOSE_CLASS }),
});
//# sourceMappingURL=toast.js.map