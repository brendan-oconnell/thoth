use uuid::Uuid;
use yew_router::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Routable)]
pub enum AppRoute {
    #[at("/login")]
    Login,
    #[not_found]
    #[at("/error")]
    Error,
    #[at("/admin/*")]
    Admin,
    #[at("/admin")]
    AdminHome,
    #[at("/")]
    Home,
}

#[derive(Debug, Clone, PartialEq, Eq, Routable)]
pub enum AdminRoute {
    #[at("/admin/dashboard")]
    Dashboard,
    #[at("/admin/works")]
    Works,
    #[at("/admin/books")]
    Books,
    #[at("/admin/chapters")]
    Chapters,
    #[at("/admin/work/:id")]
    Work { id: Uuid },
    #[at("/admin/work")]
    NewWork,
    #[at("/admin/publishers")]
    Publishers,
    #[at("/admin/publisher/:id")]
    Publisher { id: Uuid },
    #[at("/admin/publisher")]
    NewPublisher,
    #[at("/admin/institutions")]
    Institutions,
    #[at("/admin/institution/:id")]
    Institution { id: Uuid },
    #[at("/admin/institution")]
    NewInstitution,
    #[at("/admin/imprints")]
    Imprints,
    #[at("/admin/imprint/:id")]
    Imprint { id: Uuid },
    #[at("/admin/imprint")]
    NewImprint,
    #[at("/admin/contributors")]
    Contributors,
    #[at("/admin/contributor/:id")]
    Contributor { id: Uuid },
    #[at("/admin/contributor")]
    NewContributor,
    #[at("/admin/serieses")]
    Serieses,
    #[at("/admin/series/:id")]
    Series { id: Uuid },
    #[at("/admin/series")]
    NewSeries,
    #[at("/admin/publications")]
    Publications,
    #[at("/admin/publication/:id")]
    Publication { id: Uuid },
    #[at("/admin/publication")]
    NewPublication,
    #[not_found]
    #[at("/admin/error")]
    Error,
}
