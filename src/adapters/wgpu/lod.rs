//! Det grove detaljnivået til modellene langt unna.

use crate::domain::geometry::Mesh;

/// Modeller med færre trekanter enn dette får ikke noe grovt detaljnivå.
pub const LOD_MIN_TRIS: usize = 12_000;

/// Det grove detaljnivået, eller `None` når modellen er lett nok til å
/// tegnes helt (under `LOD_MIN_TRIS`): sammenslåingen av hjørner åpner
/// sprekker i murverket (en smal rute i en mur forsvinner, og to biter slås
/// sammen ulikt i skjøten), og eieren så himmel gjennom kryptmurene på
/// avstand (13. oktober).
pub fn far_detail(mesh: &Mesh, cell: f32) -> Option<Mesh> {
    (mesh.indices.len() / 3 >= LOD_MIN_TRIS).then(|| mesh.clustered(cell))
}
