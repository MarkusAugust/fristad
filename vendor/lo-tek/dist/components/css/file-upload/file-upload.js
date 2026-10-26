import { attributes, fieldStates, isFieldState, } from "../shared.js";
export const FILE_UPLOAD_CLASS = "lt-file-upload";
/** The list of the selected files. */
export const FILE_UPLOAD_LIST_CLASS = "lt-file-upload-list";
/**
 * The attributes for a file upload field.
 *
 * The field is a plain `<input type="file">`. The button inside is the
 * browser's own, and its text comes from the operating system's language. It
 * cannot be translated, so the label above the field must say what to upload.
 *
 * `accept` is a help in the file picker, not a barrier. The user can choose
 * "all files", so the type must be checked on the server anyway.
 *
 * ```ts
 * <input {...fileUpload({ multiple: true, accept: ".pdf" })} />
 * ```
 */
export const fileUpload = Object.assign(({ state = "default", multiple, accept, } = {}) => attributes({
    class: FILE_UPLOAD_CLASS,
    type: "file",
    multiple: multiple ? true : undefined,
    accept,
    "data-state": state === "default" ? undefined : state,
    "aria-invalid": state === "invalid" ? "true" : undefined,
}), {
    /** The class on the list of selected files. */
    list: FILE_UPLOAD_LIST_CLASS,
    states: fieldStates,
    isState: isFieldState,
});
//# sourceMappingURL=file-upload.js.map