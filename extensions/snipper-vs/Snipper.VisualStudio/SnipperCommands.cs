using System.Threading;
using System.Threading.Tasks;
using Microsoft.VisualStudio.Extensibility;
using Microsoft.VisualStudio.Extensibility.Commands;
using Microsoft.VisualStudio.Extensibility.Shell;
using static Snipper.VisualStudio.SnipperCommands;

namespace Snipper.VisualStudio
{
    [VisualStudioContribution]
    internal class ScaffoldConstructorCommand : SnipperCommandBase
    {
        public ScaffoldConstructorCommand(VisualStudioExtensibility extensibility)
            : base(extensibility, ScaffoldConstructorId) { }

        public override CommandConfiguration CommandConfiguration => new("%Snipper.Commands.ScaffoldConstructor.DisplayName%")
        {
            Placements = new[] { CommandPlacement.KnownPlacements.ExtensionsMenu },
            Icon = new(ImageMoniker.KnownValues.AddItem, IconSettings.IconAndText),
        };
    }

    [VisualStudioContribution]
    internal class ScaffoldPropertyCommand : SnipperCommandBase
    {
        public ScaffoldPropertyCommand(VisualStudioExtensibility extensibility)
            : base(extensibility, ScaffoldPropertyId) { }

        public override CommandConfiguration CommandConfiguration => new("%Snipper.Commands.ScaffoldProperty.DisplayName%")
        {
            Placements = new[] { CommandPlacement.KnownPlacements.ExtensionsMenu },
            Icon = new(ImageMoniker.KnownValues.Property, IconSettings.IconAndText),
        };
    }

    [VisualStudioContribution]
    internal class ImplementInterfaceCommand : SnipperCommandBase
    {
        public ImplementInterfaceCommand(VisualStudioExtensibility extensibility)
            : base(extensibility, ImplementInterfaceId) { }

        public override CommandConfiguration CommandConfiguration => new("%Snipper.Commands.ImplementInterface.DisplayName%")
        {
            Placements = new[] { CommandPlacement.KnownPlacements.ExtensionsMenu },
            Icon = new(ImageMoniker.KnownValues.Interface, IconSettings.IconAndText),
        };
    }

    /// <summary>Base for all Snipper commands.</summary>
    internal abstract class SnipperCommandBase : Command
    {
        protected SnipperCommandBase(VisualStudioExtensibility extensibility, string commandId)
            : base(extensibility)
        {
            _ = commandId;
        }

        public override Task ExecuteCommandAsync(IClientContext context, CancellationToken cancellationToken)
        {
            _ = context;
            _ = cancellationToken;
            return Task.CompletedTask;
        }
    }
}
